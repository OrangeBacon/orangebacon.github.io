---
title: Programming Language Design
date: 2026-05-13
template: ./templates/post.html
---

For a long time I've been interested in programming language development, I want to create a new language to explore development of type systems.

I have previously focused on two main areas of programming languages, the back end (optimisation and code generation) and the front end (parsing), however have never really properly experimented with type systems.  I have spent so much time thinking about the language and parsing that I've never managed to get far enough into a project.

I have decided that the best way for me to get to the type system parts of a language is to use an existing language that is easy to parse, and then add a type system.  I have chosen lua for this, as there is very little to the grammar of lua, compared with a lot of other languages, but lua still contains lots of constructs that would be interesting to write a type system for.

There are many other typed lua projects that exist currently (luau, teal, too many others to name), this one will likely be worse than all of those for pretty much everyone.  That said, there are several things I intend to do differently to the other ones.  I intend to base this project around lua 5.5, where as a lot of other projects use lua 5.1.  I would also like to be able to support types for co-routines, which are often left out.

Using lua 5.5 has several differences compared with 5.1, in particular the functions `setfenv`/`getfenv` not being present.  Instead, the variables `_ENV` and `_G` exist, which hopefully will be possible to include within the type system, unlike with the fenv functions.  Additionally, there is support for 64bit integers, separately to floating point values and a library of bitwise operators for them.

I am not intending on supporting all of base lua, there are some functions that are very hard to allow for in a type system, for example functions to allow dynamic code loading (`dofile`, `load`, etc.).  I intend for my compiler to have access to the full program to allow for whole program compilation.

Other languages which add a type system to existing dynamic languages aim to create a system where the types can easily be stripped from the language, to get working code (e.g. typescript).  I don't intend to do this, I want to be able to influence the output and the runtime using the type system.

# Parser
With that said, I still need a parser to get lua into a syntax tree.  I have previously tried to make parsers with good error handling, fancy lexers and all kinds of features, however that is pretty much the opposite of what I intend to do here.  The parser is intended to be easy to work on, rather than good for the end user.

I'm mainly saying this because it should stop me getting caught up in writing lots of code to make nice errors, trying to make the syntax tree good for auto formatting, or accidentally trying to write a language server.  That just doesn't seem like a good use of time, given my goals of working on type systems.

# Backend
Although I could stop with the type system, it would be a bit anti-climatic to not be able to run the code. The easiest way of running code that is similar to lua, is to simply make the compile target be plain lua 5.5 code, so that is what I will initially do.  Eventually though, I would like to be able to AOT compile the lua, which could use the type system to be more efficient than always doing runtime type checks.  This would likely involve outputting llvm IR, rather than writing my own optimiser.

# Future
I will try and write some more on this project once I've actually started writing the code!