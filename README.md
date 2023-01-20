# Vamp

## Primitives

```
// Integers
0
192

// Floats
1.0
3.141592

// Strings
"Hello, Vamp!"
"\n\t\x0A"

// Tags
Blue
ConstExpr
```

Tags begin with an uppercase letter like and are used as named constants. Unlike strings, tags require only constant space to store and constant time to compare.

## Tuples

Tuples combine 0 or more values and are delimited by parentheses:

```
()              // 0-tuple, often called "nil"
(1)             // 1-tuple
("sandwich", 2) // 2-tuple
```

Tuples can contain values of any type, including other tuples:

```
(((("I'm deeply nested"))))
((), 1, "x", 5.5)
```

Tuple members can be positional or named. Positional members come before named members. The order of named members is unimportant.

```
(x: 24, y: 32)
(firstName: "Ethan", lastName: "Lynn")
(3, type: Movie, description: "string")
```

Lastly, tuples can be optionally prefixed with a single tag:

```
Person(firstName: "Ethan", lastName: "Lynn")
Node(left: (Node(left: 1, right: 2)), right: ())
Ok("Success")
Error("This is an error message!")
```

## Expressions

Everything in Vamp is an expression and all expressions can have `0` or more values. The possible values of an expression form a set of possible paths a program can take. The union of all possible program paths forms the complete behavior of a program.

An expression is delimited by curly braces.

An expression with no values is called "void" and can be denoted `{}`.

An expression with a single value like `{1}` is the same as value itself, making `{}` the natural grouping operator of the language.

A multi-valued expression introduces multiple values to the context it is evaluated within. Each values is evaluated sequentially from left-to-right.

## Variables
```
