[BITS 64]
org 0x7E00
boot2:
	jmp is_A20_on
	mov esi,fail
	mov ebx,0xb8000
.loop:
	lodsb
	or al,al
	jz halt
	or eax,0x0F00
	mov word [ebx], ax
	add ebx,2
	jmp .loop
halt:
	cli
	hlt
is_A20_on:
    mov edi,0x112345  ;odd megabyte address.
    mov esi,0x012345  ;even megabyte address.
    mov [esi],esi     ;making sure that both addresses contain diffrent values.
    mov [edi],edi     ;(if A20 line is cleared the two pointers would point to the address 0x012345 that would contain 0x112345 (edi))
    cmpsd             ;compare addresses to see if the're equivalent.
    jne A20_on        ;if not equivalent , A20 line is set.
    ret               ;if equivalent , the A20 line is cleared.
A20_on:
	mov esi,hello
	mov ebx,0xb8000
	jmp boot2.loop
hello: db "Hello Mr.Midnight!!",0
fail: db "A20 is not enabled!!",0