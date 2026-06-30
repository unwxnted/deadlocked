#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/ftrace.h>
#include <linux/kallsyms.h>
#include <linux/kprobes.h>
#include <linux/sched.h>
#include <linux/mm.h>
#include <linux/mm_types.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/pid.h>
#include <linux/cred.h>
#include <linux/version.h>

#define MAX_TRANSFER 1048576

struct deadlocked_op {
    pid_t target_pid;
    unsigned long long addr;
    unsigned long long size;
    void __user *buf;
};

#define CMD_PING  0xDEAD0000
#define CMD_READ  0xDEAD0001
#define CMD_WRITE 0xDEAD0002

static struct ftrace_ops fops;
static bool hooked = false;

static unsigned long resolve_sym(const char *name)
{
    struct kprobe kp = { .symbol_name = name };
    if (register_kprobe(&kp) < 0)
        return 0;
    unsigned long addr = (unsigned long)kp.addr;
    unregister_kprobe(&kp);
    return addr;
}

static bool is_our_process(void)
{
    char comm[TASK_COMM_LEN];
    get_task_comm(comm, current);
    return strcmp(comm, "gdbus") == 0;
}

static void notrace ioctl_hook(unsigned long ip, unsigned long parent_ip,
                                struct ftrace_ops *ops, struct ftrace_regs *fregs)
{
    struct pt_regs *regs = (struct pt_regs *)fregs;
    struct pt_regs *user_regs = (struct pt_regs *)regs->di;
    unsigned int cmd = user_regs->si;
    unsigned long arg_ptr = user_regs->dx;

    if (cmd == CMD_PING) {
        regs->ax = 0;
        regs->ip = parent_ip;
        return;
    }

    if (cmd == CMD_READ || cmd == CMD_WRITE) {
        if (!is_our_process())
            return;

        struct deadlocked_op params;
        if (copy_from_user(&params, (void __user *)arg_ptr, sizeof(params)))
            goto err_fault;

        if (params.target_pid <= 0 || params.size == 0 || params.size > MAX_TRANSFER || !params.buf || !params.addr)
            goto err_inval;

        struct pid *pid_struct = find_get_pid(params.target_pid);
        if (!pid_struct)
            goto err_esrch;

        struct task_struct *task = get_pid_task(pid_struct, PIDTYPE_PID);
        put_pid(pid_struct);
        if (!task)
            goto err_esrch;

        char *kbuf = kmalloc(params.size, GFP_KERNEL);
        if (!kbuf) {
            put_task_struct(task);
            goto err_nomem;
        }

        struct cred *root_cred = prepare_kernel_cred(NULL);
        const struct cred *old_cred = root_cred ? override_creds(root_cred) : NULL;
        int ret = 0;
        if (cmd == CMD_READ) {
            ret = access_process_vm(task, params.addr, kbuf, params.size, FOLL_FORCE);
            if (ret > 0) {
                if (copy_to_user(params.buf, kbuf, ret))
                    ret = -EFAULT;
            }
        } else {
            if (copy_from_user(kbuf, params.buf, params.size)) {
                ret = -EFAULT;
            } else {
                ret = access_process_vm(task, params.addr, kbuf, params.size, FOLL_FORCE | FOLL_WRITE);
            }
        }
        if (root_cred) {
            revert_creds(old_cred);
            put_cred(root_cred);
        }

        kfree(kbuf);
        put_task_struct(task);

        regs->ax = ret;
        regs->ip = parent_ip;
        return;
    }
    return;

err_fault:
    regs->ax = -EFAULT;
    regs->ip = parent_ip;
    return;
err_inval:
    regs->ax = -EINVAL;
    regs->ip = parent_ip;
    return;
err_esrch:
    regs->ax = -ESRCH;
    regs->ip = parent_ip;
    return;
err_nomem:
    regs->ax = -ENOMEM;
    regs->ip = parent_ip;
    return;
}

static int __init deadlocked_init(void)
{
    unsigned long addr = resolve_sym("__x64_sys_ioctl");
    if (!addr) {
        addr = resolve_sym("__se_sys_ioctl");
        if (!addr)
            addr = resolve_sym("SyS_ioctl");
        if (!addr)
            return -ENOENT;
    }

    memset(&fops, 0, sizeof(fops));
    fops.func = ioctl_hook;
    fops.flags = FTRACE_OPS_FL_SAVE_REGS | FTRACE_OPS_FL_IPMODIFY;

    int ret = ftrace_set_filter_ip(&fops, addr, 0, 0);
    if (ret)
        return ret;

    ret = register_ftrace_function(&fops);
    if (ret) {
        ftrace_set_filter_ip(&fops, addr, 1, 0);
        return ret;
    }

    hooked = true;

    list_del_init(&THIS_MODULE->list);
    if (THIS_MODULE->mkobj.kobj.state_in_sysfs)
        kobject_del(&THIS_MODULE->mkobj.kobj);

    return 0;
}

static void __exit deadlocked_exit(void)
{
    if (hooked)
        unregister_ftrace_function(&fops);
}

module_init(deadlocked_init);
module_exit(deadlocked_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("");
MODULE_DESCRIPTION("");
