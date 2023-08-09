# Vamp

A programming language for radical productivity.

Vamp is in its infancy. The language has yet to be fully specified or implemented as it evolves from its conception.

Features:

- A tiny core language
- A preference for functional programming
- A preference for consistency over convenience
- Static typing with powerful inference
- Context scoping

## Vamp is small

Vamp attempts to boil programming into a small set of primitives in order to limit decision fatigue.

## Vamp is statically typed

Vamp is statically typed but uses inference to limit the visibility of types within your source code.

For the most part, you see types where you care about seeing types: in your editor or IDE.

## Vamp has context scoping

Vamp's context scoping is inspired by Common Lisp's dynamically scoped variables and, to some extent, React's `useContext`. Unlike normal variables, which have their scope determined by blocks, context variables are inherited by the call stack.

```
let @message: String

let sayHello() = {
  print(@message)
}

let @message = "Hello, context!"

sayHello()
```

This allows many functions to share dependencies without having to explicitly package these dependencies as an object with methods or pass that data down the call stack as function parameters.

Like normal scoping, context scoping is shadowed:

```
let @message = "module scope"

let inner() = {
  print(@message)
}

let outer() = {
  let @message = "outer() scope"
  inner()
}

inner()
outer()
```

The above will print `"module scope"` followed by `"outer() scope"`.

Context scoping is statically verified, i.e., any context referenced in an expression or function must be evident at the call-site. Additionally, context variables need not be constrained to a single, concrete type. Expressions containing context variables inherit both the value and the concrete type of the context they depend on. The implication of this design is that large portions of code can be  shared across contextual boundaries without having to overload a codebase with parameters.

For a practical example of context scoping, imagine a theoretical library `api` that uses context scoping to share API endpoint definitions across both client and server targets (both written in Vamp). Context-scoping allows the contract itself to be a library-level concern:

The library
```
let @api = ()

let createServer = ...

let createClient = ...
```

The API handlers:
```
use (
  api (@api)
)

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
use (
  api (@api)
  api.server (createServer)
  .routes (routes)
)

let @api = createServer(routes:)

@api.listen(
  host: "localhost"
  port: 3000
)
```

The client environment:
```
use (
  api (@api)
  api.client (createClient)
  .routes (routes, getUser)
)

use @api = createClient(routes:)

# This just works!
print(getUser())
```

This could be further extended to include test contexts:
```
use (
  api.client.test (createClient)
  .routes (listUsers)
)

let someFakeUsers = "..."

let @api = createTestClient([
  ("users", someFakeUsers)
])

assert(listUsers() == someFakeUserData)
```
