# ghlabel

A tool to automatically create and delete labels on GitHub Issues to match a YAML template.

When you create a new repository on GitHub, the default labels in the issue tracker are fine for basic use.
Sometimes you prefer to organize your labels differently and you like to use your own labels on every project.
Doing this manually via the web interface can be quite painful if you have a lot of repositories and/or a lot of labels.
ghlabel helps you save time by running one command to set a repository's labels to exactly those defined in your YAML template.

## Installation

Binary releases for Mac OS X and Linux are available on the [releases page](https://github.com/jimmycuadra/ghlabel/releases).

## Usage

```
ghlabel [FLAGS] --file <file> --token <token> --user <user> --repo <repo>

FLAGS:
  -d, --dry-run      Print what the program would do without actually doing it
  -h, --help         Prints help information
      --no-create    Do not create labels missing from the repo but present in the file
      --no-delete    Do not delete labels in the repo that are not in the file
  -v, --version      Prints version information

OPTIONS:
  -f, --file <file>      Path to a YAML file containing the label template
  -r, --repo <repo>      The name of the repository to apply the label template to
  -t, --token <token>    OAuth token for authenticating with GitHub
  -u, --user <user>      The name of the user or organization that owns the repository
```

Example:

```
ghlabel --file labels.yml --token abc123 --user rust-lang --repo rust
```

The file must contain an array of hashes, each with a name and a color. For
example, here is a template for a subset of the default GitHub Issues labels:

``` yaml
- name: bug
  color: fc2929
- name: duplicate
  color: cccccc
- name: enhancement
  color: 84b6eb
```

By default, every label in the file will be created (or updated, if the color
changed) on GitHub if it doesn't already exist and every label on GitHub not in
the file will be deleted. Limit this behavior with the --no-create and
--no-delete flags, respectively. No output from the program indicates there
were no changes made.

An OAuth token can be obtained from https://github.com/settings/tokens.
The token used requires the "repo" scope if the program will be run on a
private repo. Otherwise, it only requires the "public_repo" scope.

## Example

The file `labels.yml` in this repository was used as the template to create the labels in GitHub Issues.

## License

[MIT](http://opensource.org/licenses/MIT)
