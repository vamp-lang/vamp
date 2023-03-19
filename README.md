# Vamp

A programming language for radical productivity.

Note: This implementation is in its infancy.

- A tiny core language
- A preference for consistency over convenience
- Static typing with powerful inference
- Context variables

## Vamp is small

Vamp tries very hard to boil programming into a small set of primitives.

Often, this consistency means making things slightly less expressive or more limited, but we believe this is good tradeoff.

## Vamp is statically typed

Vamp is statically typed but uses inference to limit the visibility of types within your source code.

For the most part, you see types where you care about seeing types: in your editor or IDE.

## Vamp has context variables

Context variables are an concept that came from Common Lisp. Unlike normal variables, which have block-scoping, context variables are inherited by the call stack.

```
ctx @message: String

let sayHello() = {
  print(@message)
}

use @message = "Hello, context!"
sayHello()
```

This allows many functions to share dependencies without having to explicitly package these dependencies as an object with methods or pass that data down the call stack as function parameters.

Context is shadowed:

```
ctx @message: String
let outer() = {
  use @message = "Inner"
  inner()
}
let inner() = {
  print(@message)
}
use @message = "Outer"
inner()
outer()
```

The above will print `"Outer"` followed by `"Inner"`.

The key distinction of Vamp's context is that it can be statically determined. By the time an expression is evaluated, all of the associated context must be defined. Additionally, because context is determined during type checking, context variables need not be constrained to a single, concrete type. Expressions containing context variables inherit both the value and the concrete type of the context they depend on. The implication of this design is that code can be shared across contextual boundaries. This enables the development of an entirely new class of library code that supports compilation to multiple target environments, easy test mocking, and other unexplored territory.

For example, imagine a library `api` that allows you to define API functions consumable by both server and client environments. In a typical programming language, this would involved some amount of redundancy, but in Vamp, the entire contract between client and server can be a library-level concern:

```
ctx @api: API

```

The API handlers:
```
import (@api, @params) 'api'

let listUsers = @api.get("/users", || {
  "Listing all users"
})

let getUser = @api.get("/users/{id}", || {
  "Returning user {}"(@params.id)
})

let routes = [
  listUsers,
  getUsers,
]
```

The server environment:
```
import (api) 'api'
import (createServer) 'api/server'
import (routes) './routes'

use @api = createServer(routes:)

@api.listen(
  host: "localhost"
  port: 3000
)
```

The client environment:
```
import (api) 'api'
import (createClient) 'api/client'
import (routes, getUser) './routes'

use @api = createClient(routes:)

# This just works!
print(getUser())
```

This could be further extended to include test contexts:
```
import (createTestClient) 'api/client/test'
import (listUsers) './routes'

let someFakeUsers = "..."

use @api = createTestClient([
  ("users", someFakeUsers)
])

assert(listUsers() == someFakeUserData)
```

No more mocking and shimming, no more target compilation pragmas. Context-dependent execution is a language feature.

## Imperative-functional equivalence

Vamp has a unique way of representing many kinds of imperative control-flow constructs as purely functional types.

Mathematically, the control flow is extracted into a stack of Monads.

The result is a programming language that feels and performs like an imperative language, but inherits the benefits of strong, purely functional type system.
