# Pale Documentation

## Comments
Comments in Pale are easy to understand. "//" introduces a line comment, which just disregards everything to the end of the line, "{\*" introduces a block comment, and the block comment continues until a "}\*" is found. 
```
(some-lisp-code) // This is a helpful comment that describes the use of the function.

(more-lisp-code)
{* 
    This is a loooooooooooooong multi-line comment 
    It is helpful for explaining the use of the 
    code above, which is very hard to understand.
*}
```

## Statements
All Pale statements are either a singleton, which is called an atom, or an s-expression. For now, all s-expressions *must* start with a function.

Singletons look like this: `some-value` or `"some-literal-statement"`.

S-expressions look like this: `(+ 34 35)`. All parts of them must be either singletons or s-expressions. The functions are applied to the arguments *left to right.* Functions in Pale can also be treated as objects, so `some-function` is the function itself, whereas `(some-function)` is the return value after that function is called with zero arguments. Empty statements (`()`) are not allowed in Pale.

Some more examples of statements:
```
(print (+ 34 (- 40 (+  23))))
// Prints the funny number

(+ (10) 59)
// Produces an error, because "(10)" is an s-expression, and 10 is not a function that can be called.
```

## The Associative Operator `$`

Pale has a right-associative operator, which is the dollar sign (`$`). Programmers that have used Haskell might recognise this, as it operates mostly the same.

It opens a new statement, that is closed either at the end of the file or at the next closing parentheses. This means that this statement:
```
(print $ - 489 $ + 34 35)
```
Is the same as this: 
```
(print ( - 489 ( + 34 35)))
```

It's mostly used as shorthand for the long sets of parentheses that are common in Lisps.
