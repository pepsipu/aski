## Aski: Modern Assembly

[![Run on Repl.it](https://repl.it/badge/github/pepsipu/aski)](https://repl.it/@pepsipu/aski)

Aski's meant to replace Assembly's painful and unintuitive linear syntax.

```assembly
cmp rax, 0
jne .ne
mov rsi, my_text
lea rsi, [rsi+8]
mov [rsi], 0
.ne:
```

(Oh how it burns my eyes!)

It takes thorough effort to even understand what's going on here. What madness! With Aski, the program's intent becomes a lot clearer.

```rust
if $rax == 0 {
	$rsi = my_text + 8
	$*rsi = 0
}
```

Here we can clearly see `my_text` is getting null terminated at the 8th byte. So why was it so obscure in the Assembly code? Well, Assembly, due to it's linear nature, makes it difficult to see the path code execution takes and why. In addition, simple tasks (like writing a null byte to the end of a string) can end up to be way more instructions than what you'd expect. Although Aski compiles down to the same assembly code, people learning Assembly can get a feel and build an intution for Assembly by matching Aski statements with their corresponding Assembly output.

Now let's go over the semantics of language.

## Syntax

All Aski code given to the compiler is converted to Assembly to allow Assembly newbies to read it.

Aski is aimed to resemble C syntax, while aimed to execute like Assembly instructions would. This means, like assembly, you manipulate the registers yourself!

````rust
$rax = 2
````

 is fundamentally the same as

```assembly
mov rax, 2
```

Though, to keep things easy for those new to Assembly, you can also do

```rust
$rax = $rax + $rsi * 4 * 2 + $rdi
```

which will expand into

```assembly
lea rsi, [rsi*8]
add rax, rsi
add rax, rdi
```

or some equivilent.

### Comments

Comments follow traditional assembly syntax, using semicolons.

```assembly
; this is a comment!
```



### Functions

Defining a function can be done like so.

```rust
fn foo {

}
```

Return instructions are **automatically** generated at the end of the function, so don't worry about having to add them yourself!

Calling a function is also very easy!

```rust
call(foo)
```

You can also expose a function to the linker by using the `extern` keyword. You'll want to do this for your `_start` function.

```rust
extern fn _start() {

}
```

### Constants and Buffers

A constant string, number, or any other read only data can be marked with the `const` keyword. For example,

```rust
const hello = "hello word!"
```

Strings are **not** null terminated. If you'd like, you can terminate them yourselves.

Buffers are similar! use the `let` keyword.

```rust
let user_input: [byte, 32]
```

Woah, woah, woah! What's this `[byte, 32]` thing? Well, that's the integrated type system. Let's go over it really quickly.

#### Type System

Types specify the size of each unit as well as the amount of units. For example, in the previous example the size of the unit was a byte and the buffer was 32 bytes. But, if we change `byte` to `qword`, each unit will be 8 bytes long, so that'd mean the entire buffer would be 256 bytes!

```rust
let chunks: [qword, 32]
; that's 256 bytes!
```

To obtain the bytes occupied by a variable, just use the `sizeof()` function.

````rust
$rax = sizeof(chunks) - 1
; 255
````

### Control Flow

Currently, `if` statements are the only implemented control flow structure, besides `call`. Our modular compiler will allow us to `while` and `for` loops with not much more effort, but due to time constraints we're only working with `if` statements. They are similar to Rust `if` statements.

```rust
if $rax * 2 + rsi != $rdx - 3 {
  ; code here
}
```

### Inline Assembly

Something you need that Aski doesn't have? Use some inline assembly!

```assembly
#syscall
```

Just prefix with a hashtag.

## Example

Here's an example program that takes a file name and spits out it's contents!

```rust
const hello = "welcome to fs reader!"

const file_question = "what file would you like to read?"

const no_input_err = "no input given!"

let user_buf: [byte, 128]

; 64 chunks to store file data
let file_data: [qword, 64]

extern fn _start {
    $rsi = hello
    $rdx = sizeof(hello)
    call(print)

    call(menu)
}

fn menu {
    $rsi = file_question
    $rdx = sizeof(file_question)
    call(print)

    $rsi = user_buf
    $rdx = sizeof(user_buf)
    call(read_input)

    ; account for newline
    if $rax == 1 {
        $rsi = no_input_err
        $rdx = sizeof(no_input_err)
        call(print)
        call(exit)
    }

    ; null terminate input & remove newline
    $rsi = user_buf
    $rdi = $rsi + $rax - 1
    $*rdi = 0

    $rdi = user_buf
    call(open)

    ; give fd of file to rdi
    $rdi = $rax
    $rax = 0
    $rsi = file_data
    $rdx = sizeof(file_data)
    #syscall

    $rdx = $rax
    $rsi = file_data
    call(print)

    call(menu)
}

fn read_input {
    $rax = 0
    $rdi = 0
    #syscall
}

; prints the string in rsi and size in rdx
fn print {
    ; sys_write
    $rax = 1
    ; stdout
    $rdi = 1
    #syscall
}

fn exit {
    $rax = 60
    $rdi = 0
    #syscall
}

fn open {
    $rax = 2
    $rsi = 0
    $rdx = 0
    #syscall
}
```

PS: nasm is included in the repo for repl.it
