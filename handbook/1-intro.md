# 1. Intro to Vamp

Vamp is about building a conceptual framework for programming that is as simple as possible to limit decision fatigue and improve developer productivity.

As such, the core language is as small as possible.

## Numbers

Vamp's primitive number types are familiar looking from other programming languages:

```
# Integers
let decimal = 0
let hexadecimal = 0xF
let binary = 0b1010
let octal = 0o80

# Float
let pi = 3.141592
let bigNumber = 1e100
```

## Symbols

Vamp's symbol type is inherited from Lisp: 

```
let size = 'big'
let color = 'red'
let animal = 'dog'
```

Symbols of the same name have the same value, so storing and comparing symbols is cheap.

## Tuples

Tuples, delimited by parentheses `(, )`, combine positional and named access into a single data structure.

```
let clifford = (big, color, animal)
let point = (x: 10, y: 20)
```

Tuple members are accessed with `.`:

```
let size = clifford.0
let x = point.x
```

Tuples can be destructure in `let` statements:
```
let (matchedSize, matchedColor, matchedAnimal) = clifford
let (x: matchedX, y: matchedY) = point
```

Lastly, tuples provide an abbreviated syntax for K/V pairs of the form `key: key`. In both expressions and patterns, `(key:)` is interpreted as `(key: key)`:
```
let (x:, y:) = point
let newPoint = (x:, y:)
```

Tuples do not provide dynamic access to their members.

## Arrays

Arrays, delimited by `[, ]`, are homogenous, ordered, growable collections:
```
let groceries = ["oatmilk", "pancake syrup", "eggs"]
```

## Maps

...Coming soon...

## Note about commas

Most whitespace in Vamp is insignificant, however, newlines can be used in place of commas anywhere commas are needed:

```
let multilineTuple = (
  name: "Ethan"
  age: 24
  weight: 170
)

let multilineArray = [
  "eggs"
  "milk"
  "bread"
]
```

No trailing-comma vs no-trailing-comma debate in Vamp!

> Note: Vamp determines whether or not a comma should be inserted at the end of a line based on the preceding token. Lines ending in opening delimiters like `(,[,{` or operators `+,-,*,/,...` 
>
> In practice, this whitespace significance doesn't cause much ambiguity. However, when breaking expressions containing binary operators `+,-,*,/,...` across multiple lines, the operator must be placed at the end of a line to preserve the expression.
