start:
    load    debug   ; Load debug byte
    store   0xFF    ; Store in debug register
    load    char    ; Load character address
    jmp     print   ; Jump to `print`

print:
    lload   char    ; Load character at address `char`
    icjmp   end     ; Jump to end if character is '\0'
    rwrite          ; Write character
    load    char    ; Load character address
    add     one     ; Increment address
    store   char    ; Save updated address
    jmp     print   ; Loop

end:
    halt             ; End program


char:
    .data   hello   ; Address of first character in string

one:
    .data   0x01    ; Constant for incrementing the address

hello:
    .data   "Hello, world!\n"   ; String data

debug:
    .data   0x2A    ; Magic number for debug register
