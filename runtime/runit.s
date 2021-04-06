	.text
	.globl	main
main:
	endbr64
	pushq	%rbp
	movq	%rsp, %rbp
	subq	$48, %rsp
	movl	%edi, -20(%rbp)
	movq	%rsi, -32(%rbp)
	movq	%rdx, -40(%rbp)
	call	fork@PLT
	movl	%eax, -4(%rbp)
	cmpl	$0, -4(%rbp)
	jne	.L2
	movq	-32(%rbp), %rax
	leaq	8(%rax), %rcx
	movq	-32(%rbp), %rax
	addq	$8, %rax
	movq	(%rax), %rax
	movq	-40(%rbp), %rdx
	movq	%rcx, %rsi
	movq	%rax, %rdi
	call	execve@PLT
.L2:
	movl	$0, %eax
	leave
	ret
