#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/cdev.h>
#include <linux/device.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/sched.h>
#include <linux/mm.h>
#include <linux/pid.h>
#include <linux/version.h>
#include <linux/list.h>

#define DEVICE_NAME "i8042"
#define CLASS_NAME  "i8042"
#define MAX_TRANSFER 1048576

struct deadlocked_rw {
    pid_t target_pid;
    unsigned long long addr;
    unsigned long long size;
    void __user *buf;
};

#define DEADLOCKED_READ  _IOW(0xE0, 0x20, struct deadlocked_rw)
#define DEADLOCKED_WRITE _IOW(0xE0, 0x21, struct deadlocked_rw)

static dev_t dev_num;
static struct cdev cdev;
static struct class *deadlocked_class = NULL;

static int deadlocked_open(struct inode *inode, struct file *filp)
{
    return 0;
}

static int deadlocked_release(struct inode *inode, struct file *filp)
{
    return 0;
}

static long deadlocked_ioctl(struct file *filp, unsigned int cmd, unsigned long arg)
{
    struct deadlocked_rw params;
    struct task_struct *task;
    struct pid *pid_struct;
    char *kbuf = NULL;
    int ret = 0;

    if (copy_from_user(&params, (void __user *)arg, sizeof(params)))
        return -EFAULT;

    if (params.target_pid <= 0 || params.size == 0 || params.size > MAX_TRANSFER)
        return -EINVAL;

    if (!params.buf || !params.addr)
        return -EINVAL;

    pid_struct = find_get_pid(params.target_pid);
    if (!pid_struct)
        return -ESRCH;

    task = get_pid_task(pid_struct, PIDTYPE_PID);
    put_pid(pid_struct);
    if (!task)
        return -ESRCH;

    kbuf = kmalloc(params.size, GFP_KERNEL);
    if (!kbuf) {
        put_task_struct(task);
        return -ENOMEM;
    }

    switch (cmd) {
    case DEADLOCKED_READ:
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5, 7, 0)
        ret = access_process_vm(task, params.addr, kbuf, params.size, FOLL_FORCE);
#else
        ret = access_process_vm(task, params.addr, kbuf, params.size, 0);
#endif
        if (ret > 0 && copy_to_user(params.buf, kbuf, ret))
            ret = -EFAULT;
        break;

    case DEADLOCKED_WRITE:
        if (copy_from_user(kbuf, params.buf, params.size)) {
            ret = -EFAULT;
            break;
        }
#if LINUX_VERSION_CODE >= KERNEL_VERSION(5, 7, 0)
        ret = access_process_vm(task, params.addr, kbuf, params.size, FOLL_FORCE | FOLL_WRITE);
#else
        ret = access_process_vm(task, params.addr, kbuf, params.size, 1);
#endif
        break;

    default:
        ret = -ENOTTY;
        break;
    }

    kfree(kbuf);
    put_task_struct(task);
    return ret;
}

static struct file_operations fops = {
    .owner          = THIS_MODULE,
    .open           = deadlocked_open,
    .release        = deadlocked_release,
    .unlocked_ioctl = deadlocked_ioctl,
};

static int __init deadlocked_init(void)
{
    int ret;

    ret = alloc_chrdev_region(&dev_num, 0, 1, DEVICE_NAME);
    if (ret < 0)
        return ret;

    cdev_init(&cdev, &fops);
    ret = cdev_add(&cdev, dev_num, 1);
    if (ret < 0) {
        goto err_cdev;
    }

#if LINUX_VERSION_CODE >= KERNEL_VERSION(6, 4, 0)
    deadlocked_class = class_create(CLASS_NAME);
#else
    deadlocked_class = class_create(THIS_MODULE, CLASS_NAME);
#endif
    if (IS_ERR(deadlocked_class)) {
        ret = PTR_ERR(deadlocked_class);
        goto err_class;
    }

    if (!device_create(deadlocked_class, NULL, dev_num, NULL, DEVICE_NAME)) {
        ret = -ENOMEM;
        goto err_device;
    }

    list_del_init(&THIS_MODULE->list);

    return 0;

err_device:
    class_destroy(deadlocked_class);
err_class:
    cdev_del(&cdev);
err_cdev:
    unregister_chrdev_region(dev_num, 1);
    return ret;
}

static void __exit deadlocked_exit(void)
{
    device_destroy(deadlocked_class, dev_num);
    class_destroy(deadlocked_class);
    cdev_del(&cdev);
    unregister_chrdev_region(dev_num, 1);
}

module_init(deadlocked_init);
module_exit(deadlocked_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Intel Corporation");
MODULE_DESCRIPTION("i8042 keyboard controller driver");
