start:
    load    foo     ; Gets overwritten
    load    debug   ; Load debug value
    store   debug   ; No effect
    store   debug   ; No effect
    load    debug   ; No effect
    jmp     end     ; Jump to end
    jmp     start   ; Unreachable

end:
    store   0xFF    ; Store debug value
    halt            ; Stop
    halt            ; Unreachable

foo:
    .static "Heyyo!"

debug:
    .static 0x2A
