# A Little Game-Engine-That-Could Written in Rust

## A Short Tale

Once upon a time, there was a little developer that kept seeing things pop up about this great new language called Rust. After his curiosity in the language came to a head, he read [The Book](https://doc.rust-lang.org/book/) in its entirety. His skull sponge saturated with delicious knowledge, he decided he needed to make something that would let him put what he learned to the test. In the past, he used to make games and wanted to do more of that in his spare time. His brain cells wiggled and said, "Why don't you make a game in Rust?"

As he casually perused available options for technologies that would enable such a project, his brain cells wiggled harder and said, "Why don't you make a game _engine_ in Rust?" This sparked his interest. It was totally novel, he'd never done such a thing. He'd always used tools made by others. This, he thought, would truly be a challenge.

So, he set out to make a game engine. For the architecture, he drew on his familiarity with Unity and modeled the engine after their architecture: objects in the game are made up of and driven by _Behaviours_. These Behaviours house both the logic and data of specific concerns. These Behaviours define lifecycle methods that the engine's internal mechanisms would call at the appropriate times. He steeled himself, rubbed two brain cells together to make sure they were sufficiently warmed up, and started coding.

For hours and days he toiled, the compiler scolding him for his rookie mistakes. One by one he resolved the compiler errors, referring back to The Book and plumbing the depths of knowledge Stack Overflow and Reddit had to offer. After much keyboard-slapping and head-desking, he looked upon his work and saw that it was... _hideous_.

Rust is not an object-oriented language like C#. Rust did not like the things he was trying to do, and he had to do some very naughty things to get the compiler to allow his code past its discerning eye. While frustrating at times, he loved the compiler's unerring adhesion to correctness. It was forcing him to think differently, to challenge patterns he'd seen repeated over and over by himself and other experienced developers. Rust not only was telling him that he was doing something bad, but was being extremely specific in _why_ it was bad.

He made a demo game with the engine and, while the application code was pleasant to write, knowing what was going on under the hood pained him. He knew it could be better.

Through conversation and research, he chanced upon an architectural pattern used in game engines called ECS, or Entity-Component-System. In fact, he'd noticed that the existing Rust engines he had originally researched claimed to follow such an architecture. He looked deeper into this ECS and found that, well... There were very few concrete outlines out there on exactly how such an architecture should be implemented. He also didn't want to just read through the source of existing implementations, afraid that those implementations might overly influence his own. A mere mockery of a more complete solution wasn't what he set out to make, and wouldn't exercise his mind. Determined, he decided he now had a new challenge: take these vague, generic descriptions of an architecture and rewrite his little game engine. He knew he could do it. He had the technology.

He checked out his main branch, tagged it with `v0.1.0`, and then made a new branch: `overhaul-architecture`. With a few triumphant strokes of his keyboard, he deleted every file in the project. Scrapped. All of it. A fresh start.

With renewed vigor, he once again began his implementation. He laid out the groundwork of the engine, letting test-driven development guide his path. The compiler complained far less often this time as he was more experienced and this new architecture cared more about _composition and assocation_ rather than inheritance and shared ownership. He was no longer trying to force Rust to play by _his_ rules, but rather was thinking of ways by which he could play by theirs. With every new addition, he challenged himself to think differently.

After many more hours of work and a remade demo game, he looked upon what he had wrought and this time he saw that he _liked it_. The architecture was incredibly modular. Even the game's core systems like its renderer, UI, and performance analysis were all just like plugins slotted naturally into the main flow. The structure's simplicity was charming and made the approach for any addition clear. He dubbed this version `v0.2.0` and reveled in the joy of making a game with it.

Welcome to Thomas, the little game-engine-that-could.

## The Skinny

Thomas is a Rust game engine architected after the principles of ECS (Entity-Component-System). You can find some priming docs on the concepts [here](https://en.wikipedia.org/wiki/Entity_component_system). The project was made as an exercise in Rust. While it is intended to be competent in what it does, it's not trying to be some grand engine with a massive toolkit that everyone should be using. That being said, Thomas does not currently support audio and all rendering happens in the terminal. Think of something like the original Dwarf Fortress. This was done partially because the terminal allowed me to spend less time researching 2D and 3D rendering techniques and gave me more time to focus on what I actually made the project for: practicing my Rust skills and diving into ECS.

Before we get too far in, I'd just like to give a shoutout to Sander Mertens, the creator of [Flecs](https://github.com/SanderMertens/flecs). As I mentioned in my little tale, I avoided looking at exact implementation examples, but I found Mertens' blog posts and documentation very helpful on understanding some of the finer points of ECS.

## Key Differences from Some Other ECS Solutions

While you may have seen other ECS solutions out there like Bevy and Flecs, Thomas has some key differences to those:

### Systems are a bit more than just functions

A principle of ECS is that all game logic happens within systems. Systems have access to and know which components to operate on via a query that runs against whatever internal mechanism the engine has provided for representing the game world in memory. With queries being so tightly associated to systems, Thomas sees queries as being intrinsically tied to systems, so `System`s as you'll see them in Thomas are `struct`s that hold their `Query`s as well as the function that operates on the results of those `Query`s. This inherently keeps related information together.

### Queries are evaluated at runtime

While that statement taken out of context seems obvious, it stands in contrast to a solution like Bevy. While I'm very impressed with Bevy's type-fu, the fact that its queries are constructed via types means that it can only work with information available at compile-time. While we lose some compile-time type safety by evaluating queries at runtime, we put the responsibility of _getting_ data squarely on the shoulders of our queries, which includes filtering potential matches based on conditions. If queries were all compile-time, you would have to query for more than you actually want and then have your system filter those results by what it cares about.

For example, what if I want to write a renderer system that's going to render all the entities that are _currently_ on the screen? That system is going to want all entities with some `Renderer` component and some `Transform` or `Position` component, but only where the component with positional data puts them in a world position that's currently on the screen. You can refine your query in such a way by using `Query::has_where` in Thomas.

Giving queries the ability to filter puts responsibilities where they belong and lets `System`s focus more on their logic and less on deciding which results to actually act on. While it seems we sacrifice some type safety at first glance, the fact that Thomas encourages you to colocate your systems and queries makes it difficult to accidentally misuse the results of a query and cause a runtime error.

## A Simple Example

Everything in Thomas starts by making a `Game`. You can chain many things off the game instance before eventually `start`ing it to get the main game loop going. Since Thomas uses ECS, your game is essentially just a collection of `Component`s associated to `Entity`s, and you can manipulate the data on those `Component`s with `System`s.

That description and the following examples are extremely simplified versions of what to expect in an ECS environment. For a more complete look at what a game in Thomas might look like, see the demo I made [here](https://github.com/mrCamelCode/space_invaders). It's a little game like Space Invaders.

### Starting the game

```rust
use thomas::{Game, GameOptions, Renderer, TerminalRendererOptions, Dimensions2d};

Game::new(GameOptions {
  // You can always press Ctrl+C to stop the game.
  press_escape_to_quit: false,
  // A value of 0 here indicates an uncapped framerate.
  max_frame_rate: 30
}).start(Renderer::Terminal(TerminalRendererOptions {
  screen_resolution: Dimensions2d::new(10, 30),
  include_default_camera: true,
}));
```

If you run that, you'll just get a blank screen in the terminal that's not doing anything. That's because we haven't put anything in the game! We can change that by adding an entity to the world, like an entity that represents our player. Since our player is something that should be in the game from the start, we can add the player to the world as part of the `init` event. We'll do that by adding a `System` to the `init` event. All systems are just added to events the engine uses internally to control the flow of system invocation. While there are more events available, most application code will use the `init`, `update`, and `cleanup` events.

`init` systems run exactly **one time** before the main game loop starts.

After that, the game runs all `update` systems on every frame. When the main game loop is exited by receiving a `GameCommand::Quit` command, all `cleanup` systems are invoked before the process finally finishes.

`cleanup` should be used to undo any system side effects your game may have caused, or to do something on exit, like save the player's progress. Note that if the engine crashes, `cleanup` won't run.

### Adding the player to the world

```rust
use thomas::{Game, GameOptions, Renderer, TerminalRendererOptions, Dimensions2d, System, GameCommand, Layer, IntCoords2d, TerminalRenderer, TerminalTransform};

Game::new(GameOptions {
  press_escape_to_quit: false,
  max_frame_rate: 30
})
.add_init_system(System::new(vec![], |_, commands| {
  commands.borrow_mut().issue(GameCommand::AddEntity(vec![
    Box::new(TerminalRenderer {
      display: 'A',
      layer: Layer::base(),
      foreground_color: None
      background_color: None,
    }),
    Box::new(TerminalTransform {
      coords: IntCoords2d::new(1, 2),
    }),
  ]));
}))
.start(Renderer::Terminal(TerminalRendererOptions {
  screen_resolution: Dimensions2d::new(10, 30),
  include_default_camera: true,
}));
```

This will add a new `System` to the `init` event. `System::new`'s first argument is the list of `Query`s we want to run for this system to have access to certain components in the world. Since we don't need access to any existing components, we just give it an empty vector.

The second argument of `System::new()` is a function that takes two arguments. We don't need to use the first argument of the function for this system, so we won't talk about it just yet.

The second argument gives us access to the game's command queue. When we want to do something that modifies the state of the _world_ and not of a component that already exists in the world, like adding/modifying/destroying an entity, we can issue a command to perform that change.

As you can see, we use `GameCommand::AddEntity` to put a new entity into the world. Because an entity is logically just a collection of components, the only thing we have to give `AddEntity` is a `Vec<Box<dyn Component>>`. We add the `TerminalRenderer` and `TerminalTransform` components to the player because these are the components required to render an entity on the screen. The renderer must know where your entity is and what it looks like to draw it!

To avoid any confusion, keep in mind that since we're issuing a command to create the entity, it's not done synchronously. With `commands.issue()`, you're just queueing a command. Nothing _actually_ changes in the game world until the engine processes the command. Commands are processed after event triggers, so you can rest assured any issued commands will be processed in the frame they were issued.

If you run that, you should now see your player sitting a little bit away from the top left corner of the screen. While that's neat, it's not terribly interactive, and games should be interactive! For the final part of our little example, let's add a `System` to the `update` event to process user input to move our player around.

### Processing user input to move the player

```rust
use thomas::{Game, GameOptions, Renderer, TerminalRendererOptions, Dimensions2d, System, GameCommand, Layer, IntCoords2d, Query, Keycode, TerminalCamera, TerminalRenderer, TerminalTransform, Input};

Game::new(GameOptions {
  press_escape_to_quit: false,
  max_frame_rate: 30
})
.add_init_system(System::new(vec![], |_, commands| {
  commands.borrow_mut().issue(GameCommand::AddEntity(vec![
    Box::new(TerminalRenderer {
      display: 'A',
      layer: Layer::base(),
      foreground_color: None
      background_color: None,
    }),
    Box::new(TerminalTransform {
      coords: IntCoords2d::new(1, 2),
    }),
  ]));
}))
.add_update_system(System::new(vec![
  Query::new().has::<TerminalTransform>().has_no::<TerminalCamera>(),
  Query::new().has::<Input>(),
], |results, _| {
  if let [movables_results, input_results, ..] = &results[..] {
    let input = input_results.get_only::<Input>();

    for movable_result in movables_results {
      let mut transform = movable_result.components().get_mut::<TerminalTransform>();

      if input.is_key_down(&Keycode::A) {
        transform.coords += IntCoords2d::left();
      } else if input.is_key_down(&Keycode::D) {
        transform.coords += IntCoords2d::right();
      } else if input.is_key_down(&Keycode::W) {
        transform.coords += IntCoords2d::down();
      } else if input.is_key_down(&Keycode::S) {
        transform.coords += IntCoords2d::up();
      }
    }
  }
}))
.start(Renderer::Terminal(TerminalRendererOptions {
  screen_resolution: Dimensions2d::new(10, 30),
  include_default_camera: true,
}));
```

Whew, look at that! Let's take it apart.

```rust
vec![
  Query::new().has::<TerminalTransform>().has_no::<TerminalCamera>(),
  Query::new().has::<Input>(),
]
```

This defines our queries. This `System` needs access to the transforms in the world so we can move them around. For now, you can ignore the `has_no` clause. It's a way to exclude entities from a query's match that have the specified component.

Our system also needs access to the user's input. `Input` is a component that's injected into the world for you by Thomas (it also injects `Time`). You can query for `Input` whenever you need to read the user's input, and you can be confident your query for it will always yield a result.

Now that our system has the data it's going to work with, let's look at its logic:

```rust
|results, _| {
  if let [movables_results, input_results, ..] = &results[..] [
    let input = input_results.get_only::<Input>();

    for movable_result in movables_results {
      let mut transform = movable_result.get_mut::<TerminalTransform>();

      if input.is_key_down(&Keycode::A) {
        transform.coords += IntCoords2d::left();
      } else if input.is_key_down(&Keycode::D) {
        transform.coords += IntCoords2d::right();
      } else if input.is_key_down(&Keycode::W) {
        transform.coords += IntCoords2d::down();
      } else if input.is_key_down(&Keycode::S) {
        transform.coords += IntCoords2d::up();
      }
    }
  ]
}
```

Now we get to see the first argument of a `System`'s function! It's a `Vec<QueryResultList>`. Each entry in the vector represents the results of that query. As you can see from the names in our destructuring, the results from the queries are guaranteed to be in the same order as we defined our queries. While you're free to get at the results of your queries any way you prefer, this destructuring pattern is one that I've used that I like. It keeps the code for getting at the results succinct and readable.

Moving on, we see the line:

```rust
let input = input_results.get_only::<Input>();
```

`QueryResultList::get_only` is a convenience method that's great for queries that will always return exactly **one** result. In our case, `Input` is like a service. It should only ever have a single instance in the game at any given time, and should always exist. To save ourselves some needless for-loopery or `find`ing to get at the `Input` instance, we can just `get` the `only` result and have Thomas pull the `Input` from the list of matched components on the result.

Next, we have:

```rust
for movable_result in movables_results
```

In this case, our `movables_results` is a `Vec<QueryResultList>` where we're _not_ guaranteed to always only have a single result. The query currently yields all `TerminalTransform`s in the world (so long as the entity doesn't also have a `TerminalCamera`). While our player is currently the only thing that will match this query, that's unlikely to stay true for very long as our game develops and we add more things like NPCs. Because of that, this `System` should operate on all the results of the query.

Every element in a `QueryResultList` is a `QueryResult`. A `QueryResult` represents a _single_ entity that matched the constraints specified by the query. The `QueryResult` also contains references to the component instances on that entity that we asked for in our `Query`. In our case, we're looking through the world for all entities with a `TerminalTransform` component, so every result will represent a _different_ entity and each result will have _that_ entity's reference to `TerminalTransform` available to use. To see how this plays out, try adding another entity to the world in our `init` system that also has the `TerminalTransform` component.

Next, we have the line:

```rust
let mut transform = movable_result.get_mut::<TerminalTransform>();
```

We want to get the `TerminalTransform` component on this `QueryResult`, and because we intend to move it around based on user input, we'll have to mutate the `coords` found on the `TerminalTransform`. Therefore, we want to get a _mutable_ reference to the `TerminalTransform` component off this `movable_result`. If we only wanted to read data off the component, we might instead use `get`.

Note that both `get` and `get_mut` will panic if the component you're asking for doesn't exist in the results. In our case, we're looping over all matches, so this would only occur if we messed up and tried to pull a component off of the results that we never queried for! For example, if we tried to do:

```rust
let renderer = movable_result.get::<TerminalRenderer>();
```

The application would panic. While the entity we matched on would also have a `TerminalRenderer` in our particular case, we didn't query for that component, so it's not in our results. That's an important detail to keep in mind: when your query produces a match, it only makes the components you _asked for_ available. It does _not_ give you access to all the components on that particular entity.

If you don't like the assertiveness of `get` and `get_mut`, you can use their safer alternatives: `try_get` and `try_get_mut`. However, it should be noted that a panic from `get` and `get_mut` indicate a problem with either your `Query` or `System` which should be corrected. If you receive a panic from one of those methods, you're trying to operate on something you're not querying for. In this case, you should either update your `Query` to include the component, or correct the `System` to only operate on what it's querying for. Because of this tendency to reveal incorrect systems/queries, you should generally prefer using `get` and `get_mut`.

Finally, the meat of the `System`'s logic:

```rust
if input.is_key_down(&Keycode::A) {
  transform.coords += IntCoords2d::left();
} else if input.is_key_down(&Keycode::D) {
  transform.coords += IntCoords2d::right();
} else if input.is_key_down(&Keycode::W) {
  transform.coords += IntCoords2d::down();
} else if input.is_key_down(&Keycode::S) {
  transform.coords += IntCoords2d::up();
}
```

This is pretty straightforward, and I'm sure you can tell what's going on here, so we'll just discuss the points that might not be immediately obvious.

`is_key_down` tells us if the provided key was pressed _this_ frame. That means it will only return `true` on the single frame that it was pressed. As it stands, our user would have to keep pressing the movement keys to move around. Try changing it to `is_key_pressed` to see how it behaves differently!

You may be confused why pressing W takes us `down`, and pressing S takes us `up`. In the terminal, the origin is the top left corner. Coordinates increment from there. Therefore, in the terminal, if you want to go toward the bottom of the screen, you're actually going _up_ in coordinates. Going toward the top of the screen is going _down_.

As a side note, Thomas supports a `TerminalCamera` which can be moved around since it has a `TerminalTransform` component, so screen positions are not always the same as world positions. In fact, you already told Thomas to add the default camera when you said `include_default_camera: true`. The default camera is placed at the origin with a complete view of the screen. Currently, Thomas only supports one camera and it must be marked as the main camera. There _must_ be such a camera included in the world for the renderer to render anything. Including the default camera sets this up for you.

Why not try experimenting with moving the _camera_ around instead of the player? The `has_no` clause we ignored earlier is what's stopping our system from moving the camera around already! Try to find a way to tweak the query such that it excludes the player's `TerminalTransform` and gets only the camera's `TerminalTransform`.

And that's it! Those are the basics of Thomas, but there are plenty of other things the engine provides you to help you make a game quickly and easily. For example, our little game here has a few problems:

1. We're currently putting all our logic into the creation of our `Game`. As the game grows, this file will become nigh unreadable! Breaking your logic out into separate units you can organize more easily can be done with `SystemsGenerator`s.
2. Our movement system currently operates on _every_ entity that has a `TerminalTransform` (and no `TerminalCamera`). In a real game, that's unlikely to be what we want. We probably only want to move the player around. You could try to refine the query to make sure only the player is ever matched on, or you could make your own custom `Component` that behaves like a marker to make writing queries that match on just the player entity easier. You might also consider using the `Identity` component.
3. If we change our movement `System` to use `is_key_pressed` to give our player's finger a break, everything might seem okay at first until you change the `max_frame_rate` of the game. You may notice that our movement is framerate-dependent. Ew! What is this, Dark Souls? We'd probably want to make a custom `Component` that keeps track of a `Timer` that lets us control how quickly translations can be applied to our player. Maybe that's a useful piece of information to include on a custom `Player` component?

To see examples of all these techniques and more (like UI and collisions) in action, check out the [demo](https://github.com/mrCamelCode/space_invaders) I mentioned earlier! It's a simple game inspired by Space Invaders.
