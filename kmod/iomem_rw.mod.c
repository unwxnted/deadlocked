#include <linux/module.h>
#include <linux/export-internal.h>
#include <linux/compiler.h>

MODULE_INFO(name, KBUILD_MODNAME);

__visible struct module __this_module
__section(".gnu.linkonce.this_module") = {
	.name = KBUILD_MODNAME,
	.init = init_module,
#ifdef CONFIG_MODULE_UNLOAD
	.exit = cleanup_module,
#endif
	.arch = MODULE_ARCH_INIT,
};



static const struct modversion_info ____versions[]
__used __section("__versions") = {
	{ 0xad20279a, "get_pid_task" },
	{ 0xe28760b4, "register_ftrace_function" },
	{ 0xcb8b6ec6, "kfree" },
	{ 0x4131ebd0, "find_get_pid" },
	{ 0xd272d446, "__fentry__" },
	{ 0x79d3714a, "__put_cred" },
	{ 0xbd03ed67, "__ref_stack_chk_guard" },
	{ 0xd272d446, "__stack_chk_fail" },
	{ 0xff0106da, "refcount_warn_saturate" },
	{ 0x9479a1e8, "strnlen" },
	{ 0x7e0199db, "access_process_vm" },
	{ 0xd70733be, "sized_strscpy" },
	{ 0xdd77a1de, "ftrace_set_filter_ip" },
	{ 0xe28760b4, "unregister_ftrace_function" },
	{ 0x0a798924, "const_current_task" },
	{ 0x4f1e5fd0, "__list_del_entry_valid_or_report" },
	{ 0xe54e0a6b, "__fortify_panic" },
	{ 0xd272d446, "__x86_return_thunk" },
	{ 0x092a35a2, "_copy_to_user" },
	{ 0xb9fcd065, "call_rcu" },
	{ 0xc8b64d54, "kobject_del" },
	{ 0xe5179ac5, "validate_usercopy_range" },
	{ 0xb6377019, "register_kprobe" },
	{ 0x2de0a194, "unregister_kprobe" },
	{ 0x9ad558f4, "prepare_kernel_cred" },
	{ 0x1cf09ab5, "__put_task_struct_rcu_cb" },
	{ 0xab5df0a5, "put_pid" },
	{ 0xa61fd7aa, "__check_object_size" },
	{ 0x092a35a2, "_copy_from_user" },
	{ 0xd710adbf, "__kmalloc_noprof" },
	{ 0x90a67d58, "module_layout" },
};

MODULE_INFO(depends, "");


MODULE_INFO(srcversion, "98277A36CA2B895AD84B9E6");

MODULE_INFO(suserelease, "openSUSE Tumbleweed");
