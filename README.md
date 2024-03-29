# kindle-hl2fr

> Convert Kindle highlights to Org or JSON

Parses Kindle `My Clippings.txt` file and prints deduplicated and sorted highlights from selected documents either in the Org format (quote blocks) or in JSON. A convenience shell script is included that uses [fzf](https://github.com/junegunn/fzf/) to streamline the process of document selection.

`kindle-hl2fr` is only meant to support the author’s particular use case. If you are looking for a more broad solution, try e.g. [Fyodor](https://github.com/rccavalcanti/fyodor/).

Binaries are not provided.

## Table of contents

- [Usage](#usage)
- [Installation](#installation)
- [Setup](#setup)
- [License](#license)

## Usage

(This presupposes steps specified in [Installation](#installation) and [Configuration](#configuration).)

Execute the convenience script passing the path to your `My Clippings.txt` as an argument:

````
$ hl2fr.sh "./examples/My Clippings.txt"
````

`fzf` will display a list of document titles present in the file. Select either just one of them by navigating to its list entry and pressing `Enter` or multiple ones by marking them with `Tab` and then pressing `Enter` to confirm. The highlights from selected documents should be printed to `stdout`.

### Direct usage

````
$ kindle-hl2fr

Usage: kindle-hl2fr <My Clippings file> {{org|json}} [Document titles]
````

Running with only the first argument will return titles of all documents that have entries in the specified file:

````
$ kindle-hl2fr "./examples/My Clippings.txt"

La Possibilité d’une île (Houellebecq, Michel)
Nienasycenie (Witkiewicz, Stanisław Ignacy)
Old Man Goriot (Balzac, Honoré de)
````

The second and third arguments are for the actual highlight printing mode. The second argument specifies the desired output format and the third specifies the titles of the documents whose highlights should be printed:

````
$ echo -e "La Possibilité d'une île (Houellebecq, Michel)\nOld Man Goriot (Balzac, Honoré de)" | xargs -0 kindle-hl2fr "./examples/My Clippings.txt" "org"

DOCUMENT: La Possibilité d'une île (Houellebecq, Michel)
#+begin_quote
This is also highlight.
#+end_quote

DOCUMENT: Old Man Goriot (Balzac, Honoré de)
#+begin_quote
This is a highlight.
#+end_quote
````

The Org formatter performs some additional modifications of the highlight’s content:

````
$ kindle-hl2fr "./examples/My Clippings.txt" "org" "Nienasycenie (Witkiewicz, Stanisław Ignacy)"

DOCUMENT: Nienasycenie (Witkiewicz, Stanisław Ignacy)
#+begin_quote
[…] sentence number 1, open at both ends […]
#+end_quote
#+begin_quote
Sentence number 2, closed at both ends.
#+end_quote
#+begin_quote
[S]entence number 3, open at the start and closed at the end!
#+end_quote
#+begin_quote
Sentence number 4, closed at the start and open at the end […].
#+end_quote
#+begin_quote
<This
is
a
test
note>
#+end_quote
…
````

Compare the above with the content of [`/examples/My Clippings.txt`](/examples/My%20Clippings.txt).


JSON output example:

````
$ echo -e "La Possibilité d'une île (Houellebecq, Michel)\nOld Man Goriot (Balzac, Honoré de)" | xargs -0 kindle-hl2fr "./examples/My Clippings.txt" "json"

[
  {
    "clippings": [
      {
        "content": "This is a highlight.",
        "kind": "highlight",
        "locationOrPage": [
          10,
          14
        ]
      }
    ],
    "documentTitle": "Old Man Goriot (Balzac, Honoré de)"
  },
  {
    "clippings": [
      {
        "content": "This is also highlight.",
        "kind": "highlight",
        "locationOrPage": [
          24,
          27
        ]
      }
    ],
    "documentTitle": "La Possibilité d'une île (Houellebecq, Michel)"
  }
]
````

## Installation

Requirements:

* Rust stable.

Run:

````
$ cargo install --git "https://github.com/fauu/kindle-hl2fr"
````

or, if installing from within the cloned repository directory:

````
$ cargo install --path .
````

See also:

````
$ cargo install --help
````

## Setup

If you wish to use the convenience script, install [fzf](https://github.com/junegunn/fzf/), make `scripts/hl2fr.sh` executable and set the variable `kindle_hl2fr_path` inside it to reflect the path of the `kindle-hl2fr` executable file built during installation. If you need JSON rather than Org output, modify the `output_format` variable as well.

## License

See [COPYING.md](COPYING.md).
