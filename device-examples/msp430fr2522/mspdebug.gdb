target remote :2000

set print asm-demangle on

break panic
break main

load
monitor reset
