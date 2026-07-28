---
title: Name Resolution
date: 2026-07-28
template: ./templates/post.html
intro: Processing an abstract syntax tree into a more useful format.
---

Now we have a syntax tree from the parser (See [Part 1](https://orangebacon.github.io/posts/2026-05-26-parser)), we need to do something with it.  The current tree is not the easiest to work with directly for any type checking, and there are many things that could be improved.

The first, and most immediately noticeable thing that could be changed is the lifetime parameters.  In the AST, we stored tokens directly from the lexer, which referred directly to the source text.  Although this helped reduce memory allocations, it has made the tree harder to store and harder to work with.  I have therefore chosen to allocate strings where required during name resolution, to remove this parameter.

# String Interning
The easiest option for allocating strings is to convert each `&str` into a `String` and store that directly in the tree.  This, however requires calling the allocator for each string.  Additionally, strings are 24 bytes (at least on the 64bit platforms I use), which would make the new tree quite big if used everywhere.  Finally, to check if strings are equal is O(n), which can get slow with a lot of strings.

Instead, I am 'interning' the strings.  I create a list of all strings used in the tree, then whenever I want to include a string in the tree, I put the index into that list.  I'm using `u32` to store the index, as it is unlikely that anyone will exceed 2^32 strings in a single file.  I then use a hash map to convert from a String into its ID.  Therefore, equality on strings is as simple as checking if the string ids are equal, rather than checking the whole length of the string.

This uses these fields stored within the name resolution pass:
```rs
map: HashMap<String, StringId>
string_table: Vec<String>
```

There are significantly more efficient string interning methods than this[^1], however they are more complex, this simple method will work for now.

[^1]: My string interner allocates way more than it should, with lots of separate allocations which could be stored as a single `char` array.  I also allocate each string twice, once for the hash map and once for the string table, however this could be reduced.  See [Matklad's Blog post on interning](https://matklad.github.io/2020/03/22/fast-simple-rust-interner.html) as an example.

# Literal resolution
Within the lua source code, string literals can contain escape sequences such as `\n` for representing line breaks, `\012` to represent the decimal code point numbered `012` and many more.  These escape sequences make writing string literals easier, however need translating into their actual value before the string can be used.  I chose to implement this translation within the name resolution pass as it allows the strings to be interned as explained above.

A string literal in lua can contain pretty much any data.  I restrict all source code files to use utf-8, however escape sequences can represent any byte, regardless of whether it is valid utf-8.  I decided to store all strings as `Vec<u8>`, so that these string literals can be stored correctly.  I store all strings as vecs, even ones directly in the source code that I know are in utf-8.

Number literals were also converted into their actual numeric representation.  This uses [lua's rules](https://www.lua.org/manual/5.5/manual.html#3.1) for whether a number is an integer or a float, and then stores the parsed number in the resolved tree.

The number representation uses a custom wrapper around floating point numbers, so they can be sorted using `f64::total_order`, unlike the default implementation in rust, which doesn't implement `Ord` at all.

# Name resolution
Name resolution is the most complex part of this pass (as you'd expect, given its name).  I had a couple of attempts at getting a good system that achieves what I wanted, all of which had different compromises.

## Locals
Local variables were relatively easy.  I implemented local variable resolution pretty much identically to how it is done in [Crafting Interpreters](https://craftinginterpreters.com/local-variables.html), using a stack of local variables.

## Globals
Global variables, however are pretty different to locals, and lua's globals are particularly complex.  In their simplest form, a global variable access in lua is equivalent to looking up a name within the current environment, i.e. `my_global = 5` is the same as `_ENV["my_global"] = 5`.  Lua also has a `global` statement which is used for defining global variables and attributes on them, e.g. `global<const> a` declares `a` to be a constant global variable and `global<const> *` declares all globals to be constant.

### `_ENV`
The environment that a global variable is looked up in is initially defined by the compiler, before the start of any user code.  However, there is nothing stopping someone writing directly to `_ENV`, or declaring a local variable which shadows `_ENV`.

If I define the function:
```lua
local count = 0
local function set_env()
    return setmetatable({print = print}, {
        __index = function(table, key)
            count = count + 1
            print(count)
            return "a"
        end
    })
end
```

Then the below code, prints `1 2 3`.
```lua
local _ENV = set_env()
a = not_found
a = help
b = owo
```

If the existing value of `_ENV` is modified, as in the below code:
```lua
local function p() print(not_found) end
_ENV = set_env()
p() -- 1 a
```
Then the environment changes within the function, as it is changing the value of the environment, even though the memory location which is read within the function doesn't change.

Alternatively, if a new environment is declared e.g.:
```lua
local function p() print(not_found) end
local _ENV = set_env()
p() -- nil
```
Then then the function hasn't had its environment modified, so is still using the variable for the environment that was in scope when it was declared.  Another way of thinking about this is as if every function captures the `_ENV` value when it is defined, making all functions closures.

All this is relatively un-intuitive, but can allow for some pretty powerful patterns, e.g. sandboxing, logging, and weird code only useful to confuse people.  As dynamic as it seems, this is significantly less dynamic than the previous system (`setfenv`/`getfenv`), which allowed changing the environment of a function after it had been declared.

Either way, I intend to support `_ENV`, and its dynamic nature.

### `global`
Global statements were introduced in lua 5.5 to help prevent errors with undefined global variables.  In particular, if you miss-spell the name of a local variable, it can just assign a value to a new global, or read a nil value, all without ever emitting an error to help with the mistake.

The statement `global *` declares that all unknown variables are globals.  The statement `global<const> *` declares that all unknown variables are globals that cannot be assigned to (but can be read from).  By default, every file has an implicit `global *` at the start, which was the original behaviour, before global statements were introduced.

More interesting are the global statements that don't end in `*`, for example `global a`.  The main effect of this is to declare `a` to be a global variable, however it additionally invalidates any `global *` statements.  This means that any globals not introduced in following global statements will produce an error at compile time.  This overriding includes overriding the implicit `global *` at the start of every file.

If a global statement has an initialiser e.g. `global a = 5`, then it checks if the global variable is nil.  If so, it sets the value of the global, otherwise a run-time error is thrown.

The global statements are all lexically scoped.  This means that you can type one within a block and it will only effect the code within that block.  As an example:
```lua
do
    global a = 5
    print(a)
end
print(b)
```
Will print `5 nil`, but not error, as the implicit `global *` is only overridden within the block, but it is in scope at the `print(b)` statement.

### Combination of the 2
The 2 global variable systems are both quite different.  Lua by default uses both at the same time, which means that a `global` statement can contradict what would be possible using `_ENV`.

```lua
global<const> a = 5
local _ENV = {}
a = 3 -- cannot assign to a constant
```
In this program, the `<const>` attribute is applied to the first environment and once it has been replaced, the attribute still applies, but to a completely different variable.

```lua
global a = 5
global print
print(b) -- error
local _ENV = { b = 5, print = print }
print(b) -- still an error because of the lexical scope of `global`
```
In this one, the first `global` statement means that un-defined globals will cause an error.  However, by changing the environment, I define `b`, but it wasn't using a global statement, so it still errors at compile-time, even though the variable does exist at run-time.

Note that I had to include `global print` in the file cannot even see the standard library of lua!

### What was Implemented
The simplest solution, in my opinion was to simply ignore a lot of the effects of a `global` statement.  A global statement has 3 main effects:

- Assign a value to a global variable (if a value was given)
- If that assignment was to a global that isn't `nil`, emit an error at runtime
- The lexical scoping effects as shown above

I just decided to keep the first 2 effects and not bother with the 3rd.  This means that `global *`, `global<const> *`, `global a` and `global<const> a` all have no effect, as no value is assigned.

This seems to be the simplest solution to the problem, in my opinion.  It doesn't try to mislead people that a `global` statement actually effects anything about variable resolution.  I should probably add a warning message to remind users of this, however errors are still implemented as `panic!`, let alone trying to implement warnings.

It doesn't help anything about making typos in local variable names, but hopefully a type system would help with that, without requiring the lexical scoping and name resolution effects of global statements.

# Attributes
The attribute syntax in lua is (at least to me) very ugly.  I am hoping that I can add new features to the language that prevent having to use attributes (at least with this syntax).

However, I still think that they should be supported in the parser and name resolver, even if there are better ways to write the same things, so here is how I implemented the 2 default lua attributes:

## `<const>`
The `<const>` attribute was pretty easy to implement entirely within the name resolution pass.  `<const>` means that the variable cannot be modified, however it is just a syntax marker, rather than actually changing anything about the variable itself.

When a variable is declared, I check whether it has the `<const>` attribute, and store that within the variable table.  Then, any time an assignment statement is parsed, I check whether it is assigning directly to a const variable.  This means that an assignment `a[5] = 3` is always allowed, regardless of whether `a` is const or not.

## `<close>`
In order to implement locals with the `<close>` attribute, I recorded where the scope of each local variable starts and ends.  For variables introduced as function parameters or loop variables, this is trivial to calculate from the name tree without recording it.  These variables also cannot have attributes applied to them.  However, for variables defined in a `local` statement, it is less trivial.  I therefore added `ScopeStart(VariableID)` and `ScopeEnd(VariableID)` statements to the tree for each local variable.

The scope ends are not stored in the tree as statements, they are stored as a separate list of variable IDs which need closing, for all times a scope ends.  To show why its a separate list, consider this example:
```lua
local file<close> = io.open("example.txt")
return file:read(10)
```
We have the following order of operations:
- open file
- read file and record the return value
- `file` goes out of scope, so call its `__close` method
- return the previously recorded value

In order to order the call to `__close` and the return statement correctly, the list of variables to be closed needs to be recorded separately to the rest of the statements.

To make this more complex, consider a repeat-loop:
```lua
repeat
	local file<close> = io.open("example.txt")
until file:read(15)
return
```
This (nonsensical) example should have the following order:
- enter loop
	- open file
	- read the file and record its return value
	- `file` goes out of scope, so call its `__close` method
	- if the return value was truthy, continue in the loop
- return

This therefore requires knowledge of the variable scopes within the implementation of the repeat loop, so that the condition is checked at the right time relative to the end of the scope of the variables in the loop.

# Goto resolution
While resolving variables, I also did the name resolution for goto statements.  A goto is able to jump to any label within the current function, regardless of whether it is before or after the goto.

Within the name tree, each goto and label is stored as an ID.  Each goto statement contains the ID of a label, and each label statement contains its own ID.

If a goto is found, I try looking up the label in a stack of already found labels. if in there, then a goto statement for that label is emitted.  Otherwise, a new label is added to the stack and the goto statement goes to that undefined label.  At the end of the name resolution pass, it is checked that all labels are defined, and if not an error is emitted.

Each label stores the nesting level of the function that it was defined in, and when a function definition finishes, the gotos from that function are popped off the top of the stack.

Similar to goto, lua has a `break` statement.  This exits the current loop and allows control flow to continue in the statement after the loop.  This is the same as having a label immediately after a loop and using goto within the loop, so I transform all uses of break into this goto and label.  When a loop is being analysed, I add a new label to the stack of defined labels, then any break statement can iterate through the stack to find that label.  The iterator through the stack ensures that the closest loop is found, and checks that the loop is in the same function as the break statement.  When the loop ends, I then emit the target for those gotos and remove the label from the goto stack.  This does however mean every loop counts as 2 statements.

Goto does have a restriction that it is not allowed to jump over the declaration of a variable.  This hasn't been checked within the name resolution pass, and will need to be verified later in the compiler.  Jumping over variable declarations doesn't matter for globals, as they are equivalent to indexing into a table, as above.  It does, however, matter for local variables, which will be undefined if their declaration is jumped over.  In the original lua implementation (PUC lua), this could result in reading any arbitrary value off the stack, or reading off the end of the stack and causing a segmentation fault.  Due to the analysis we're doing to the code, it would be possible to insert a definition of local variables (i.e. set them to `nil`) if the declaration is jumped over, however pretty much every time this is done will be a user error, so I think its better to throw a compiler error and not silently accept it.

# What I learnt while writing this
I kept changing the name tree structure throughout writing the resolver.  I'm glad that I didn't try writing a pretty printer for the tree until I'd finished writing the pass as every change to the tree would be significantly more complicated.

This did have the downside, however, of me having to look at debugging outputs showing `Vec<u8>` e.g. `[95, 69, 78, 86]` instead of Strings, e.g. `_ENV`.

I likely should implement some kind of testing for the compiler.  Throughout writing the lexer, parser, and now name resolver I've been using small test programs and looking at the output by hand.  I should likely have a system for running test files and comparing their expected outputs.

I also learnt a lot more about lua than I knew before.  Even if you think you know something, I've found that actually trying to write code to implement that thing is significantly harder and required you to think much deeper than if you just use the feature.

Additionally I have made this project slightly incompatible with base lua 5.5.  Some of the incompatibilities that I am aware of are:
- As said above, global variable declarations work differently.
- This project is a compiler, I'm not planning on implementing run-time code loading, so no `eval`-like functions.  This also has effects on how the `require` function works.
- Lua lets you catch a lot of error messages and look at them as string values, I am not going to try to keep them the same.  (Not least as a type system will move a lot of errors around!)
- Lua checks for out of memory errors, stack overflow errors, and attempts to sandbox code.  I do not, there is no security boundary between user code if using this implementation.  Instead, I would prefer to compile the user code to run in some other sandboxed environment, e.g. wasm.

I originally wanted to be fully compatible, but have since decided that there really is no point, and I would just be making my life harder.  Therefore my lua is *mostly* compatible, with the differences probably not mattering to most programs.

# Result
The name resolution pass that we've implemented, is a combination of all of the above transformations into one pass over the AST.  This should be a lot nicer to work with moving forwards, as we know that all of the above transformations were successful.

There are still several constructs in this resultant tree that can easily be simplified.

Method calls in lua can be converted into an equivalent function call:
```lua
a:c()
```
is equivalent to:
```lua
a.c(a)
```
Converting between these 2 reduces the amount of constructs that a type checker has to deal with, so should make things easier for us in the future.

Making changes like this will be done in the next pass, consuming the tree produced by the name resolution pass, as there is quite a bit more complexity than just the simple transformation above.

I also intend to write more about the design direction and future plans for the language, at some point, potentially before writing the next pass? (who knows? not me)

The code for the name resolution we've gone through is in [the repository](https://github.com/OrangeBacon/typed-lua/tree/da7f280174e100a3be3be196840658565051828d).  This includes a few further transformations that are required to fully run the name resolution.  A list of all the transformations made is included at the top of the `name_tree.rs` file.