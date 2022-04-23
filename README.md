# dicer-rs
Dicer-rs originated as a discord bot intended to roll dices for table-top RPGs. \
While this is still a valid use case, dicer-rs is now a fully fledged programming language
featuring a minimalistic syntax and a target on array manipulation and TTRPG automation.
## Syntax
`f D 7d6 {$D+1}` iterates overs the results of 7 throws of 6 sided dices and add one before
outputting the result \
`V=1 wV<100{$[0]xV V=V+1}` outputs a 98 lines long triangle of 1 in arrays \
Spaces are optional but greatly improve readability. \
`f` starts a 'foreach' loop : the syntax is f V A { ... } \
`w` starts a 'while' loop : the syntax is w C { ... } \
`i` starts an 'if else' structure : the syntax is i C { ... } e { ... } \
`$` outputs the following expression \


