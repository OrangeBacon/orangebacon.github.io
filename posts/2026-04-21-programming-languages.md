---
title: Choosing a Programming Language
date: 2026-04-21
template: ./templates/post.html
---
I thought I would give a few opinions about some of the programming languages I have used (even if a little briefly).[^1]

[^1]: Yes the code blocks are here to test the syntax highlighter in the static site generator oops!

# Rust
```rust
fn main() {
  println!("Hello, World!");
}
```
Rust is the main language I use, and is the one that I have the most experience with, so I am likely biased towards using it.  A lot of the code I have written is command-line applications, which run directly on the user's computer.

Rust doesn't work particularly well for web development in my opinion.  It can be used easily via web assembly, which I think would be good for isolated parts of websites, however using anything that isn't HTML & CSS for a website is, in my opinion, the wrong approach.

I like how strict the type system is, using type driven design has helped me get code right more often than I did with a less strict language.  I also enjoy things like pattern matching and how the language encourages good code architecture as it is pretty much required by the borrow checker.  This does make it slower to write though, especially for experiments.

However, a lot of libraries for game development and graphics are all oriented around c++, which makes some projects that I would be interested in a lot harder to write in rust.

# C++
```c++
#include <print>
void main() {
  std::println("Hello, World!");
}
```
I've used c++ before, however the initial reason I moved away from it was having to deal with cmake.  Having an easy to use build system would make c++ a lot more attractive to me, however I haven't tried more recent cmake versions, and I have heard that there have been improvements within the last couple of years.

C++ is a very complex language and there are lots of places where I have and will likely make mistakes that are hard to see unless you have a lot of experience with c++, especially in memory safety and correctness of code in edge cases.  Rust solves a lot of these issues for me, however some parts of c++ can be more expressive than rust, for example specialisation and generics vs templates.

# C
```c
#include <stdio>
void main() {
  printf("Hello, World!");
}
```
I used to write code in C, but no longer feel like it is the right language for me to be using all the time.  I did enjoy the simplicity of C, but that is also a downside to me, in that it is harder to write more complex code, in particular generic code is nearly impossible without macros or lots of `void*`.  The code is also going to be longer, given the lack of abstractions, which makes it more time consuming and error-prone to use, however it is easier to know exactly what is happening in the code.  C is also required for lots of embedded code, as there are lots of microcontrollers that only have C toolchains and aren't supported by other languages.

# Zig
```zig
const std = @import("std");

pub fn main() void {
    std.debug.print("Hello, World!\n");
}
```
Zig seemed like a cool language, but not the right one for me, when I tried it.  The language is very explicit about pretty much everything, including allocators and all control flow, which is much better than c is in my opinion.

I feel like zig is too low level of a language for me though, you can still easily segfault or cause undefined behaviour like in c and c++, where as it is much harder in (safe) rust.  I enjoy programming, itself, but I don't especially enjoy the debugging and how much harder it is to debug a segfault compared with a logic issue.

The `comptime` parts of zig seem to me to be both really useful and add a significant amount of complexity to the language.  It is hard to know exactly what inputs are required for each function, with all the type based code being dynamically typed at compile time. This makes it harder to work out if the code is correct, like with any dynamically typed language.  I also don't like how this means that code that isn't run, isn't type checked, unlike a lot of other statically typed languages. (I am aware that most of these apply to c++ too)

# C#
```c#
class Program {
  static void Main() {
    System.Console.WriteLine("Hello, World");
  }
}
```
I think that C# has come a long way since last time I used it, in particular with the dotnet cli, AOT compilation, actual cross platform support, and now pattern matching (yes its been many years).  It wouldn't be suitable for cli development as much due to the large binary sizes (due to including a garbage collector, etc) and slower start time than a native language, but for scripting tasks such as in game development, C# seems like a really good language.

# JavaScript
```js
console.log("Hello, World!");
```
I've used JavaScript because I have to for websites, however it isn't my favourite language.  I generally prefer static typing, which JS very much isn't.  The biggest issue I have, however, with JS is its ecosystem.  Everything about compiling and using JS is very complicated and slow, with JS compilation potentially taking longer than native code generation.  So many different tools and frameworks are used, all of which seem really complicated and come with their own learning points and more tooling, which generally make it very hard to actually get started with a JS application without already knowing what you are doing.

# Typescript
```ts
console.log("Hello, World!");
```
Typescript solves some of the issues I have with JS, notably with the lack of static typing, yet makes the ecosystem issue worse.  TS has to be compiled and checked, it cannot be run directly so it is yet another layer of tooling to manage and understand.

Additionally, the types are all erased at run-time, and can potentially be wrong.  The type system is really easy to get around and pass wrong types.  It also has very complicated types which can be even harder to understand than the code they annotate.

I can see how TS is useful for a team which has a big codebase with an already existing build system and lots of collaboration, yet I don't think it is the right thing for me as an individual.

# Conclusion
This honestly doesn't conclude in any particular way with languages being "good" or "bad", instead they all have different use cases and I would potentially use many different languages depending on where they are being used.
