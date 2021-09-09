# `shadergarden`

Shadergarden is a tool for building hot-code-reloadable shader pipelines. For a tutorial for how to get started, consult the [introductory blog post](https://blog.tonari.no/shadergarden) or the [shadergarden lisp language documentation](./LISP.md).

## Usage
Once you've installed shadergarden via `cargo install shadergarden`, test to see that it is installed properly by running:

```
shadergarden --help
```

This should print out some usage information. To create a new project, run:

```
shadergarden new path/to/project
```

This will create a new example project in the specified directory. To run a shadergarden, cd into the directory of a project and run:

```
shadergarden run
```

This should open a new window and start running your graph. Don't close the window if you want to make changes; instead, open the project in an editor of your choice - the graph will update on save.

If a build error is encountered while reloading, `shadergarden` will log the error and continue executing the old graph.

### Fancier Usage
You can pass input images and videos to shadergarden using the `-i` flag. This flag takes a list of paths to photos/videos - you must pass the same number of input photos/videos as the number of `(input ...)`s specified in `shader.graph`.

Once you've got a nice shadergarden, to render out a png sequence, use the `render` subcommand. This subcommand works exactly the same as `run`, but requires an output directory. To render the game of life demo out into a gif, run:

```
mkdir out
shadergarden render demos/life -o out -s 30 -e 430
ffmpeg -i "out/frame-%4d.png" -framerate 30 life.gif
```

You should see something like this (it might be a *little* fancier):

<p align="center">
    <img src="./demos/life/life.gif">
</p>

Happy hacking!
