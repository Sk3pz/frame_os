%define FREE_SPACE 0x9000

ORG 0x7C00
BITS 16

; Main entry point where BIOS leaves us.

Main:
    jmp 0x0000:.FlushCS               ; Some BIOS' may load us at 0x0000:0x7C00 while other may load us at 0x07C0:0x0000.
                                      ; Do a far jump to fix this issue, and reload CS to 0x0000.

.FlushCS:
    xor ax, ax

    ; Set up segment registers.
    mov ss, ax
    ; Set up stack so that it starts below Main.
    mov sp, Main

    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    cld

    mov ax, 0x2401
	int 0x15
	mov [disk],dl
	mov ah, 0x2    ;read sectors
    mov al, 1      ;sectors to read
    mov ch, 0      ;cylinder idx
    mov dh, 0      ;head idx
    mov cl, 2      ;sector idx
    mov dl, [disk] ;disk idx
    mov bx, boot2;target pointer
    int 0x13
    ; Point edi to a free space bracket.
    mov edi, FREE_SPACE
    ; Switch to Long Mode.
    jmp SwitchToLongMode


BITS 16
%include "assembly/bootloader/mbr/includes/LongModeDirectly.asm"
BITS 16
; Prints out a message using the BIOS.

; es:si    Address of ASCIIZ string to print.

Print:
    pushad
.PrintLoop:
    lodsb                             ; Load the value at [@es:@si] in @al.
    test al, al                       ; If AL is the terminator character, stop printing.
    je .PrintDone
    mov ah, 0x0E
    int 0x10
    jmp .PrintLoop                    ; Loop till the null character not found.

.PrintDone:
    popad                             ; Pop all general purpose registers to save them.
    ret
disk:
	db 0x0

; Pad out file.
times 510 - ($-$$) db 0
dw 0xAA55
[BITS 64]
boot2: