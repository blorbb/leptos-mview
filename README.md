# Leptos mview

[![crates.io](https://img.shields.io/crates/v/leptos-mview.svg)](https://crates.io/crates/leptos-mview)

<!-- cargo-rdme start -->

An alternative `view!` macro for [Leptos](https://github.com/leptos-rs/leptos/tree/main) inspired by [maud](https://maud.lambda.xyz/).

This crate is still very new and probably has loads of bugs as my first attempt at proc macros - please open an issue if you find any!

## Example

A little preview of the syntax:

```rust
use leptos::*;
use leptos_mview::mview;

#[component]
fn MyComponent() -> impl IntoView {
    let (value, set_value) = create_signal(String::new());
    let red_input = move || value().len() % 2 == 0;

    mview! {
        h1.title { "A great website" }
        br;

        input
            type="text"
            data-index=0
            class:red={red_input}
            prop:{value}
            on:change={move |ev| {
                set_value(event_target_value(&ev))
            }};

        Show
            when=[!value().is_empty()]
            fallback=[mview! { "..." }]
        {
            Await
                future=[fetch_from_db(value())]
                blocking
            |db_info| {
                p { "Things found: " strong { {*db_info} } "!" }
                p { "Is bad: " f["{}", red_input()] }
            }
        }
    }
}

async fn fetch_from_db(data: String) -> usize { data.len() }
```

<details>
<summary> Explanation of the example: </summary>

```rust
use leptos::*;
use leptos_mview::mview;

#[component]
fn MyComponent() -> impl IntoView {
    let (value, set_value) = create_signal(String::new());
    let red_input = move || value().len() % 2 == 0;

    mview! {
        // specify tags and attributes, children go in braces
        // classes (and ids) can be added like CSS selectors.
        // same as `h1 class="title"`
        h1.title { "A great website" }
        // elements with no children end with a semi-colon
        br;

        input
            type="text"
            data-index=0 // kebab-cased identifiers supported
            class:red={red_input} // non-literal values must be wrapped in braces
            prop:{value} // shorthand! same as `prop:value={value}`
            on:change={move |ev| { // event handlers same as leptos
                set_value(event_target_value(&ev))
            }};

        Show
            // values wrapped in brackets `[body]` are expanded to `{move || body}`
            when=[!value().is_empty()] // `{move || !value().is_empty()}`
            fallback=[mview! { "..." }] // `{move || mview! { "..." }}`
        { // I recommend placing children like this when attributes are multi-line
            Await
                future=[fetch_from_db(value())]
                blocking // expanded to `blocking=true`
            // children take arguments with a 'closure'
            // this is very different to `let:db_info` in Leptos!
            |db_info| {
                p { "Things found: " strong { {*db_info} } "!" }
                // bracketed expansion works in children too!
                // this one also has a special prefix to add `format!` into the expansion!
                //    {move || format!("{}", red_input()}
                p { "Is bad: " f["{}", red_input()] }
            }
        }
    }
}

// fake async function
async fn fetch_from_db(data: String) -> usize { data.len() }
```

</details>

## Purpose

The `view!` macros in Leptos is often the largest part of a component, and can get extremely long when writing complex components. This macro aims to be as **concise** as possible, trying to **minimise unnecessary punctuation/words** and **shorten common patterns**.

## Performance note

Currently, the macro expands to the [builder syntax](https://github.com/leptos-rs/leptos/blob/main/docs/book/src/view/builder.md) (ish), but it has some [performance downsides](https://github.com/leptos-rs/leptos/issues/1492#issuecomment-1664675672) in SSR mode. I may write up an alternative expansion that uses this SSR optimization, but it doesn't exist for now (feel free to contribute to this feature if you would like!).

## Compatibility

This macro will be compatible with the latest stable release of Leptos.

| `leptos_mview` version | Compatible `leptos` version |
| ---------------------- | --------------------------- |
| `0.1.0`                | `0.5.0`-`0.5.1`             |
| `0.2.0`                | `0.5.2`                     |

## Syntax details

### Elements

Elements have the following structure:

1. Element / component tag name (`div`, `App`).
2. Any generics where applicable.
3. Any classes or ids prefixed with a dot `.` or hash `#` respectively.
4. A space-separated list of attributes and directives (`class="primary"`, `on:click={...}`).
5. Either children in braces (`{ "hi!" }`) or a semi-colon for no children (`;`).
    - If the element is last in the block, no semi-colon is needed. This is mainly to make it easier to write, as an invalid macro removes syntax highlighting/autocomplete. It is advised to always add a semi-colon to the end if no children are required.

Example:
```rust
mview! {
    div.primary { strong { "hello world" } }
    input type="text" on:input={handle_input};
    MyComponent data=3 other="hi";
}
```

Adding generics is the same as in leptos: add it directly after the component name, without the turbofish `::<...>`.
```rust
#[component]
pub fn GenericComponent<S>(ty: PhantomData<S>) -> impl IntoView {
    std::any::type_name::<S>()
}

#[component]
pub fn App() -> impl IntoView {
    mview! {
        GenericComponent<String> ty={PhantomData};
        GenericComponent<usize> ty={PhantomData};
        GenericComponent<i32> ty={PhantomData};
    }
}
```

Note that due to [Reserving syntax](https://doc.rust-lang.org/edition-guide/rust-2021/reserving-syntax.html),
the `#` for ids must have a space before it.
```rust
mview! {
    nav #primary { "..." }
    // not allowed: nav#primary { "..." }
}
```

### Slots

[Slots](https://docs.rs/leptos/latest/leptos/attr.slot.html) ([another example](https://github.com/leptos-rs/leptos/blob/main/examples/slots/src/lib.rs)) are supported by prefixing the struct with `slot:` inside the parent's children.

Using the slots defined by the [`SlotIf` example linked](https://github.com/leptos-rs/leptos/blob/main/examples/slots/src/lib.rs):
```rust
use leptos::*;
use leptos_mview::mview;

#[component]
pub fn App() -> impl IntoView {
    let (count, set_count) = RwSignal::new(0).split();
    let is_even = MaybeSignal::derive(move || count() % 2 == 0);
    let is_div5 = MaybeSignal::derive(move || count() % 5 == 0);
    let is_div7 = MaybeSignal::derive(move || count() % 7 == 0);

    mview! {
        SlotIf cond={is_even} {
            slot:Then { "even" }
            slot:ElseIf cond={is_div5} { "divisible by 5" }
            slot:ElseIf cond={is_div7} { "divisible by 7" }
            slot:Fallback { "odd" }
        }
    }
}
```

### Values

There are (currently) 3 main types of values you can pass in:

- **Literals** can be passed in directly to attribute values (like `data=3`, `class="main"`, `checked=true`).
    - However, children do not accept literal numbers or bools - only strings.
        ```rust
        mview! { p { "this works " 0 " times: " true } }
        ```

- Everything else must be passed in as a **block**, including variables, closures, or expressions.
    ```rust
    mview! {
        input
            class="main"
            checked=true
            madeup=3
            type={input_type}
            on:input={move |_| handle_input(1)};
    }
    ```

    This is not valid:
    ```rust
    let input_type = "text";
    // ❌ This is not valid! Wrap input_type in braces.
    mview! { input type=input_type }
    ```

- Values wrapped in **brackets** (like `value=[a_bool().to_string()]`) are shortcuts for a block with an empty closure `move || ...` (to `value={move || a_bool().to_string()}`).
    ```rust
    mview! {
        Show
            fallback=[()] // common for not wanting a fallback as `|| ()`
            when=[number() % 2 == 0] // `{move || number() % 2 == 0}`
        {
            "number + 1 = " [number() + 1] // works in children too!
        }
    }
    ```

    - Note that this always expands to `move || ...`: for any closures that take an argument, use the full closure block instead.
        ```compile_error
        # use leptos_mview::mview;
        # use leptos::logging::log;
        mview! {
            input type="text" on:click=[log!("THIS DOESNT WORK")];
        }
        ```

        Instead:
        ```rust
        mview! {
            input type="text" on:click={|_| log!("THIS WORKS!")};
        }
        ```

The bracketed values can also have some special prefixes for even more common shortcuts!
- Currently, the only one is `f` - e.g. `f["{:.2}", stuff()]`. Adding an `f` will add `format!` into the closure. This is equivalent to `[format!("{:.2}", stuff())]` or `{move || format!("{:.2}", stuff())}`.

### Attributes

#### Key-value attributes

Most attributes are `key=value` pairs. The `value` follows the rules from above. The `key` has a few variations:

- Standard identifier: identifiers like `type`, `an_attribute`, `class`, `id` etc are valid keys.
- Kebab-case identifier: identifiers can be kebab-cased, like `data-value`, `an-attribute`.
    - NOTE: on HTML elements, this will be put on the element as is: `div data-index="0";` becomes `<div data-index="0"></div>`. **On components**, hyphens are converted to underscores then passed into the component builder.

        For example, this component:
        ```rust
        #[component]
        fn Something(some_attribute: i32) -> impl IntoView { ... }
        ```

        Can be used elsewhere like this:
        ```rust
        mview! { Something some-attribute=5; }
        ```

        And the `some-attribute` will be passed in to the `some_attribute` argument.

- Attribute shorthand: if the name of the attribute and value are the same, e.g. `class={class}`, you can replace this with `{class}` to mean the same thing.
    ```rust
    let class = "these are classes";
    let id = "primary";
    mview! {
        div {class} {id} { "this has 3 classes and id='primary'" }
    }
    ```

    See also: [kebab-case identifiers with attribute shorthand](#kebab-case-identifiers-with-attribute-shorthand)

Note that the special `node_ref` or `ref` or `_ref` or `ref_` attribute in Leptos to bind the element to a variable is just `ref={variable}` in here.

#### Boolean attributes

Another shortcut is that boolean attributes can be written without adding `=true`. Watch out though! `checked` is **very different** to `{checked}`.
```rust
// recommend usually adding #[prop(optional)] to all these
#[component]
fn LotsOfFlags(wide: bool, tall: bool, red: bool, curvy: bool, count: i32) -> impl IntoView {}

mview! { LotsOfFlags wide tall red=false curvy count=3; }
// same as...
mview! { LotsOfFlags wide=true tall=true red=false curvy=true count=3; }
```

See also: [boolean attributes on HTML elements](#boolean-attributes-on-html-elements)

#### Directives

Some special attributes (distinguished by the `:`) called **directives** have special functionality. All have the same behaviour as Leptos. These include:
- `class:class-name=[when to show]`
- `style:style-key=[style value]`
- `on:event={move |ev| event handler}`
- `prop:property-name={signal}`
- `attr:name={value}`
- `clone:ident_to_clone`
- `use:directive_name` or `use:directive_name={params}`

All of these directives except `clone` also support the attribute shorthand:
```rust
let color = create_rw_signal("red".to_string());
let disabled = false;
mview! {
    div style:{color} class:{disabled};
}
```

The `class` and `style` directives also support using string literals, for more complicated names or multiple classes at once.
```rust
let yes = move || true;
mview! {
    div class:"complex-[class]-name"={yes}
        style:"doesn't-exist"="white"
        class:"class-one class-two"={yes};
}
```

### Children

You may have noticed that the `let:data` prop was missing from the previous section on directive attributes!

This is replaced with a closure right before the children block. This way, you can pass in multiple arguments to the children more easily.
```rust
mview! {
    Await
        future=[async { 3 }]
    |monkeys| {
        p { {*monkeys} " little monkeys, jumping on the bed." }
    }
}
```

Note that you will usually need to add a `*` before the data you are using. If you forget that, rust-analyser will tell you to dereference here: `*{monkeys}`. This is obviously invalid - put it inside the braces. (If anyone knows how to fix this, feel free to contribute!)

Summary from the previous section on values in case you missed it: children can be literal strings (not bools or numbers!), blocks with Rust code inside (`{*monkeys}`), or the closure shorthand `[number() + 1]`.

## Extra details

### Kebab-case identifiers with attribute shorthand

If an attribute shorthand has hyphens:
- On components, both the key and value will be converted to underscores.
    ```rust
    let some_attribute = 5;
    mview! { Something {some-attribute}; }
    // same as...
    mview! { Something {some_attribute}; }
    // same as...
    mview! { Something some_attribute={some_attribute}; }
    ```

- On HTML elements, the key will keep hyphens, but the value will be turned into an identifier with underscores.
    ```rust
    let aria_label = "a good label";
    mview! { input {aria-label}; }
    // same as...
    mview! { input aria-label={aria_label}; }
    ```

### Boolean attributes on HTML elements

Note the behaviour from Leptos: setting an HTML attribute to true adds the attribute with no value associated.
```rust
use leptos::view;
view! { <input type="checkbox" checked=true data-smth=true not-here=false /> }
```
Becomes `<input type="checkbox" checked data-smth />`, NOT `checked="true"` or `data-smth="true"` or `not-here="false"`.

To have the attribute have a value of the string "true" or "false", use `.to_string()` on the bool.

Especially using the closure shorthand `[...]`, this can be pretty simple when working with signals:
```rust
use leptos::*;
use leptos_mview::mview;
let boolean_signal = create_rw_signal(true);
mview! { input type="checkbox" checked=[boolean_signal().to_string()]; }
```

## Contributing

Please feel free to make a PR/issue if you have feature ideas/bugs to report/feedback :)

### Extra feature ideas

- [ ] [Extending `class` attribute support](https://github.com/leptos-rs/leptos/issues/1492)
- [ ] [SSR optimisation](https://github.com/leptos-rs/leptos/issues/1492#issuecomment-1664675672) (potential `delegate` feature that transforms this macro into a `leptos::view!` macro call as well?)
- [x] Support slots
 

<!-- cargo-rdme end -->
