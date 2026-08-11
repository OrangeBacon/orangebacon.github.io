---
title: Syntax Highlighting
date: 2026-08-11
template: ./templates/post.html
intro: Putting more effort than is reasonable into syntax highlighting
---

for some reason, i want good syntax highlighting on this website.  however, i am not especially happy with all the pre-existing solutions.  therefore, i want to implement my own syntax highlighter!  ~~one thats perfect and does everything ever and might even be the best ever~~ not really, i'll implement one that is only useful for this website in particular, and add features over time as needed.

# problems with the existing
there are many existing highlighters, but they all have some issues that make them not work quite right for me, some of these issues are:
- old syntaxes, that can't cope with new language features
- hard to add new languages
- bake the themes into the output, so css cannot change the colouring
- slow (start up or run time)
- hard to get good html output (line numbers should use numbered lists as the native html numbering solution)

therefore, i have decided to write my own one

# my syntax highlighting
in order to write a syntax highlighter, lets start by working out the inputs and outputs.

the easiest input is the source code, just copy it from the markdown file. for the language to highlight, i wanted to know both the language itself and any properties.  i therefore implemented a simple parser to get the value of the properties and the language name.

the properties were chosen to allow more control over the output code, so it can look better for someone reading the rendered code block:
- diff - if a diff is requested, the highlighter should be able to tell which lines are being added, which are being removed, and work out how to highlight the code.  most parsers would (should?) error if they encounter a diff marker in the code, so a diff handler would need to remove the markers, highlight the code and then re-construct the diff.
- line highlighting - show a yellow background for some of the lines, to make them stand out more
- hide - some of the source lines are needed to make the language parse correctly, but don't add anything to the page, so should be hidden from the reader

i can specify these properties in the header line of the code block, e.g.:
    rust diff=true highlight=1,3,4 hide=#

the output was represented per line, with a style for every length of input text:
```rust
struct Line {
    content: Vec<(Style, &str)>,
    kind: Kind,
}

enum Kind {
    None,
    Highlight,
    DiffAdd,
    DiffRemove,
}
```

# rust
syntax highlighting rust code is pretty hard.  if it is tokenised, it will produce a token tree, not a simple list of tokens.  then, if you parse that into a syntax tree, some of the token boundaries will have changed.  for example:
```rs hide=#
# fn a() {
tuple.0.1
# }
```
compared to
```rs hide=#
# fn a() {
tuple(0.1)
# }
```
has the `0.1` parsed as the tuple accessors, or as a floating point number.  we can parse rust using [syn](https://crates.io/crates/syn), which will take care of all this for us.  syn is typically used for procedural macros, however there isn't anything stopping me using it as a generic rust parser, outside of a macro.  we can then convert the token stream produced by syn into the output format described above.

converting from syn's output into something usable by a syntax highlighter was a lot more work than i expected. in particular, i had to:
- work out which line each token was on, including blank lines, new lines at the end of source code and tokens that take more than 1 line,
- recurse through the tokens (as rust's tokens are a tree)
- detect identifiers vs keywords (as the tokens don't have any difference)
- work out if there should have been a comment or whitespace between any 2 tokens
- work out if something is a variable or a type (i cheated and checked the capitalisation)

# lua
i'm using lua code a decent amount, but don't have it highlighted.  i could implement syntax highlighting using the existing parser, but that would be hard, in the same way as it is in rust.  if i wait until after ive implemented type checking, then the actual kind of each token can be represented more accurately.  i should also be able to include the syntax highlighter in the compiler, to get more consistent output in multiple different environments.

# pikchr
to create the diagrams in the site, i currently use [pikchr](https://pikchr.org/home/doc/trunk/homepage.md).  in each markdown file, the diagrams include the source of the diagram in a code block.  the input and output from pikchr is worked out completely separately from the other code blocks

# other languages
i've used several other languages so far, but only very briefly.  if i decide that those other languages need highlighting, i could use an existing syntax highlighter and work out how to put its output into this highlighter, same as i did for rust.

# overall data flow
The overall data flow of the highlighter, for all languages is:
```pikchr
linerad = 10px
linewid *= 0.5

circle radius 10%
arrow 2*arrowht
Start: diamond "Is" "Diagram?"
arrow right 150% "Yes" above
DiaOut: file "HTML+SVG" "Output" fit

arrow down from Start.s "  No" ljust
Remove: oval "Remove prefix from" "lines to be hidden" fit
arrow 50%

Diff: diamond "Is diff?" fit
arrow from previous.e then right until even with DiaOut.w "Yes" above
oval "Split into a and" "b documents" fit
arrow down 50% from previous.s
oval "Highlight a and b" "documents separately" fit
arrow down 50%
oval "Combine documents" "a, b, and diff markers" fit
Arrow: arrow down 30% then right until even with Diff then down 30%

arrow down from Diff.s "  No" ljust
oval "Highlight document" fit
arrow to Arrow.end
oval "Add whole line" "highlighting" fit
arrow down 50%
DoRemove: oval "Remove lines" "to be hidden" fit
arrow down 50%
oval "Cleanup output" fit
arrow from previous.e then right until even with DiaOut.w
DiaOut: file "HTML Output" fit

arrow left 50% from Remove.w then down until even with DoRemove then to DoRemove.w
```

# performance
the previous tree-sitter version of this website took approximately 600ms to build, every time.  when i removed tree sitter and just output empty strings instead of code blocks, that was reduced to 14ms. now it is 16ms.  as far as i can tell, most of the tree-sitter slowness was due to startup time, however as there is so little work to be done, the startup time is actually important. [^1]

[^1]: not that any of these numbers were worked out particularly scientifically, i just ran the site generator a few times by hand and looked at a rough average

# example
```rs diff=true highlight=2,4
@@ 1,2 @@ my annotation
pub struct A(bool);

impl A {
+    fn example(&mut self) -> Self { /* ... */ }
}

-fn example2(&mut self) -> bool { /* ... */ }
```

# conclusion
it was probably overkill to have done this, but it feels better?
