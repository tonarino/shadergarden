# A Brief Introduction to shadergraph Lisp
A `ShaderGraph` defines a directed graph of computation. The easiest way to create one for general use is through a small lispy configuration language. This language isn't Turing complete as it's meant for configuration - in this sense, the graph doesn't do any computation itself, it just *describes* how computation should be done.

## Getting Started
Let's look at a really basic example:

```clojure
(input  texture)
(output texture)
```

This shader graph has a single node called `texture`. All it does is immediately return the input texture. This isn't very exciting, but it works. To evaluate this shader graph from Rust, you need to hook it up to a glium context:

```rust
use shadergraph::{map, program::{graph_from_sexp, load_shaders}};

// Create the graph within a context
let context = /* create a glium context */;
let (mut graph, inputs, outputs) = graph_from_sexp(
    context,
    load_string!("..."), // shader graph source code
    load_shaders("..."), // path to shader folder
);

// Pass a texture through the graph
let texture = /* create a glium Texture2d */;
let results = graph.forward(map! {
    inputs[0] => &texture,
})
let output = results[outputs[0]];
```

First, we build a graph within a specific glium context with the `graph_from_sexp` function. This function takes a shader graph lisp configuration and a table of shaders, and returns three things: the final graph, the `NodeId`s of all the inputs, and the `NodeId`s of all the outputs.

A shader graph is made of `Node`s, and can have multiple inputs and outputs. Because the graph owns the `Node`s inside it, we pass around `NodeId`s to refer to specific `Node`s within a graph. These IDs are lightweight (internally just a number), and can be copied freely. When passing things in and out of the shader graph, you have to have handles on the right `NodeIDs`.

## Using a Shader
Now that we know how to set everything up, let's write something a bit more complex - a shader that combines multiple textures!

Shader Graph use GLSL as its shading language. It's not too hard to pick up, but if you'd like to learn more before continuing I highly recommend you read through [The Book of Shaders](https://thebookofshaders.com/) and [Inigo Quilez's Website](https://iquilezles.org/). A shader is a program that runs a simple function on each pixel in parallel to produce an output.

A shader receives inputs through *uniforms*, which can be things like time, vectors, or even textures. By convention, we use the `u_` prefix to denote uniforms.

Here's a GLSL shader that takes two input textures, and displays them split-screen.

```glsl
// split_screen.frag
#version 140

uniform sampler2D u_texture_0;
uniform sampler2D u_texture_1;

in vec2 coords;
out vec4 color;

void main() {
    if (coords.x < 0.5) {
        color = texture(u_texture_0, coords);
    } else {
        color = texture(u_texture_1, coords);
    }
}
```

(We're using an old version of GLSL for compatibility, but newer versions are supported as well.)

Let's break this down! As you can see, this shader takes two uniforms: `u_texture_0` and `u_texture_1`. In a shader graph, a shader may take N inputs, written `u_texture_0, u_texture_1, ..., u_texture_<N-1>`.

In addition to these two uniforms, there are two things present in all shaders. We have an input, `coords`, and an output, `color`. `coords` is the pixel's coordinate pair between 0 and 1, with the bottom-left corner being (0, 0). `color` is an RGBA vector that we assign to to color the output.

Let's use this shader in our shader graph now:

```clojure
(input left)
(input right)

(let combined
    (shader "split_screen" 512 512 left right))

(output combined)
```

As you can see, our graph takes two inputs, `left` and `right`, and produces a single output. This output is created by joining the inputs together in a shader! To understand how this works, let's go over the two new keywords we've introduced: `let` and `shader`.

```clojure
(let <symbol> <expression>)
```

`let` assigns the value of an expression to a symbol. In this case, the value that `(shader ...)` produces is of type `NodeId`, so `combined` must also be a `NodeId`. Inputs and outputs must also be node IDs, but other types do exist, as you'll see soon.

```clojure
(shader <name> <width> <height> <inputs...>)
```

`shader` is a bit more complicated. It's a built-in function that takes a number of arguments, and creates a shader node in the shader graph with those properties. Here's a breakdown of the arguments it expects:

- `name` is a string. If you use `load_shaders` when calling `graph_from_sexp`, each shader will be named after its file stem, i.e. `<name>.frag`.

- `width` and `height` are both natural numbers that set the resolution of the output texture in pixels. For example, a `shader` with a `width` and `height` of 300 by 100 will produce an output 300 by 100 pixels large. This output texture is used when chaining shaders together.

- `inputs...` - all trailing arguments are inputs that are passed into the shader, as `u_texture`s. Each input must be a `NodeID`, of course.

So, returning to this line:

```clojure
(let combined
    (shader "split_screen" 512 512 left right))
```

Here we create a node in the shader graph that takes two inputs and runs them through the `split_screen` shader, producing an output that is 512x512 pixels large.

## Composing Shaders
This is cool and all, but it's a bit boring. Isn't the whole point of a shader graph the ability to *compose* shaders?

Yep! Here's a slightly more complex example:

```clojure
(input image)

; define a function
(define (sharpen width height image iter)
    (let edges
        (shader "sobel" width height image))
    ; iteratively sharpen the image
    (let out image)
    (repeat iter
        (let out
            (shader "sharpen" width height edges image)))
    ; return the output
    out)

(let sharpened (sharpen 1080 1920 image 7))
(output sharpened)
```

If you're reeling right now from all the lisp, no worries. We'll break it down.

(And if you're still uncomfortable after this, you can always use Rust to build a shader graph directly.)

So, let's break it down! The first keyword new keyword we run into is `define`. `define`, quite sensibly, defines a new function for later use. It looks like this:

```clojure
(define (<symbol> <arguments...>)
    <body...>
    <output>)
```

Shader graph lisp is a lisp 2 (meaning functions and variables exist in separate namespaces), and does not support higher order functions (nor does it have any operations to operate on lists, so its uefulness as a lisp is debatable ;).

Anyway,

- `symbol` is the name of the function to be defined, and `arguments...` are the list of symbols to be bound in the new scope where `body...` and `output` are evaluated.

- `body...` is made of a number of *statements*. A statement is something like `let`, a form that doesn't produce an output.

- `define` itself is a statement, but the function it defines must be used as an *expression*, which means it must produce an output. The last item in `define`, `output`, is an *expression*. An expression is something like `(shader ...)` or a variable - i.e. a form that *does* produce an output.

I'm more of a lisp 1, 'everything is an expression' kinda guy myself, but I appreciate how separating namespaces and making the distinction between statements and expressions allows one to enforce language constraints in an elegant manner (e.g. guaranteed termination).

Onward! Inside the definition of `sharpen`, we encounter a new statement called `repeat`.

```clojure
(repeat <times>
    <body...>)
```

`repeat` is a fairly rigid thing, but it might be made more flexible in the future. All it really does is copy-paste the statements inside `body...` a bunch of times. How many times, exactly? `times` times, of course! (`times` must be a positive integer.)

We can call this function in a pretty standard way:

```clojure
(let sharpened (sharpen 1080 1920 image 7))
```

Functions can only return one argument, which is a bit of a limitation at the moment. For this reason, I suggest using functions for linear portions of the shader graph. This may be changed in the future, but I'd like to do so in a way that doesn't introduce generalized list processing.

> What I'm thinking is something like this:
> ```clojure
> (define (pair one two) [one two])
> ; destructuring
> (let [one two] (pair one two))
> ```
> But this has the potential to become complicated fairly fast. We'll see.
>
> It's important to note that this wouldn't introduce any additional overhead. These programs define a shader graph, and two exactly-the-same shader graphs, even if defined in different ways, will run with exactly the same performance.

Finally, we'll cover some of Shader Graph Lisp's more advanced features.

## Advanced Features
When writing GLSL shaders, it's common to use `#define` statements to define useful constants. For instance:

```glsl
#define SEARCH 10

for (int i = 0; i < SEARCH; i++) {
    // ...
}
```

Loops in GLSL are unrolled at compile time, which means they must have a fixed number of iterations, also known at compile time. For this reason, we *can't* set the number of iterations through a dynamic mechanism, like a uniform.

But what if you have a shader that can applied in a lot of different situations, with each situation requiring slightly different constants? For example, what if we want to `SEARCH` to be smaller at high resolutions (so we do less work), or what if we want to search *backwards* by starting at `SEARCH` and decrementing `i` in other situations?

It may sound a bit crazy, but Shader Graph Lisp comes equipped has a *preprocessor preprocessor*. Like `#define`, this *pre-preprocessor* inserts useful constants at compiletime for later use. Unlike `#define`, however, these constants can be passed in through Shader Graph Lisp.

Here's a simple example:

```glsl
<SEARCH>
<BACKWARDS>

#ifdef BACKWARDS
    for (int i = 0; i > -SEARCH; i--) {
        // ...
    }
#else
    for (int i = 0; i < SEARCH; i++) {
        // ...
    }
#endif
```

`<SEARCH>` and `<BACKWARDS>` are *prepreprocessor* hooks that will be expanded into `#define` macros. We refer to shaders that take hooks as *parameterized shaders*. If you call a shader without the right hooks in place, you'll get a compile-time error. So, how can we set up these hooks? Like this!

```clojure
(let found
    (shader-param
        ("search" 100 100 texture)
        (define "SEARCH" 10)
        (ifdef "BACKWARDS" #t)))
```

`shader-param` is an expression that loads a shader, while also replacing the prepreprocessor hooks. Here's the form it follows:

```clojure
(shader-param
    (<name> <width> <height> <inputs...>)
    <hooks...>)
```

The first form in `shader-param` defines the parameterized shader to use. If you look closely, this is same way we normally define shaders (i.e. `(shader ...)`), only without the `shader` keyword.

After this form, we list as many hooks as needed. There are two types of hooks currently supported:

1. `(define <HOOK> <value>)` takes a `HOOK`, which must be a string, and a `value`, which much be representable as a string, and expands the corresponding `<HOOK>` in glsl into `#define HOOK value`.

2. `(ifdef <HOOK> <boolean>)` also takes a hook, but it needs a boolean as well. In lisp, `#t` and `#f` are used to represent true and false, respectively. If `boolean` is true, `<HOOK>` will be expanded to `#define <HOOK> 1`. Otherwise, `<HOOK>` will be removed.

With these two mechanisms, it's possible to define parameterized shaders that work well in many different circumstances.

Define macros in GLSL also support arguments. This is beyond crazy, but I'm obliged to include it for completion:

```glsl
// channel_op.frag
// Apply an operation to a pair of pixels in each color channel.

// This will be replaced:
<OP(a, b)>

uniform sampler2D u_texture_0;
uniform sampler2D u_texture_1;

in vec2 coords;
out vec2 color;

void main() {
    vec3 first  = texture2D(u_texture_0, coords, 0.).rgb;
    vec3 second = texture2D(u_texture_0, coords, 0.).rgb;

    color = vec4(
        OP(first.r, second.r),
        OP(first.g, second.g),
        OP(first.b, second.b),
        1.);
}
```

Then, in the shader graph:

```clojure
(input first)
(input second)

; make the param shader nicer to use
; this is what abstraction is for
(define (channel_op width height op first second)
    (shader-param
        ("channel_op" width height first second)
        (define "OP(a, b)" op)))

(let w 640)
(let h 480)

; take the average of two images
(let average
    (channel_op w h "(.5*(a+b))" first second))

; take the channel-wise minimum of two images
(let minimum
    (channel_op w h "min(a, b)" first second))

; take the difference of the average and the minimum
(let difference
    (channel_op w h "(a-b)" average minimum))

(output difference)
```

Yep. Crazy, right?

## Other Node Types

> TODO

## Hot Code Reloading
Why go through the trouble of defining a new language? Any why couldn't we just use something like JSON and be done with it?

The answer to the first question is hot code reloading; the answer to the second question is ease of prototyping.

Hot code reloading allows us to define shader graphs, then rebuild them live *at runtime*. As anyone who has ever messed around with shaders before can attest, the most fun part is seeing your changes update live. By making not only the shaders, but also the pipeline in which they are embedded in reloadable, it's insanely easy (and a heck-ton of fun) to experiment with different shaders and how they work together.

In this sense, hot code reloading and ease of prototyping go hand-in-hand; the former enables the latter.

To try out hot code reloading, `cd` into `resource` and type `cargo run --release` (using `prime-run` if you have a GPU and don't want your computer to die.) This will load the shader graph specified in `shader.graph`, and begin executing it if no issues exist.

When you save after editing a shader, or the graph itself, `shadergraph` should detect your changes and recompile everything. If compilation succeeds, it'll switch out the old graph with the new; otherwise, it'll print the error and keep running the old one.

Happy hacking!

## Appendix
Here's a quick reference for common uniforms and stuff.

### Input/Output
- Input: `coords`, the coordinate of the current pixel from 0 to 1, with the bottom-left being the origin. This is a `in vec2`.
- Output: `color`, an RGBA pixel. This is a `out vec4`.

### Uniforms
- Textures: `u_texture_<N>` is the Nth texture passed into the shader. It is a `uniform sampler2D`.
- Time: `u_time` is the time, in seconds, since the shader last started running. it is a `uniform float`
- Resolution: `u_resolution` is the output resolution size, in pixels. This is a `uniform vec2`.

### Common Definitions
I thought there would be more, but:
```glsl
// the size of a pixel with respect to coords.
// i.e. coords.x + PIXEL.x is exactly one pixel over.
#define PIXEL (1.0 / u_resolution)

// ...
```
