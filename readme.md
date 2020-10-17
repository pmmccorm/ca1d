# ca1d

1d cellular automata simulation with terminal and PNG output.

Simulates 1d automata with given Wolfram style rules and codes.

Limitations (because it gets awkward to specify outside of these):
* first order
* odd sized neighborhoods
* max radix (number of symbols) is in [2,36]

There are various other limitations when things get large, the easiest to run into
being the rule number must fit into a u128. To fix this I need to implement:
```
fn parse_rule(r: String, from_radix: u32, to_radix: u32) -> Hashmap<u128, Cell>
```

There are a handful of different output modes:
* Null (evaluates and then does nothing)
* Cell (cell value to ascii digit)
* Ascii (limited radix)
* Unicode (limited radix)
* AnsiGrey (greyscale ascii)
* UnicodeAnsi (default - half height unicode + any radix)
* PNG (writes to stdout, RGB, any radix)
* Raw (writes cell values in binary, no newlines)

## examples

```
ca1d <radix> <neighborhood> <rule number> <start config string>
```

Note the start config string is padded to width and is given in base 36, which
is ordered 0..9abc...z

```
$ ca1d 2 3 30 1 --output=Ascii
                                              @
                                             @@@
                                            @@  @
                                           @@ @@@@
                                          @@  @   @
                                         @@ @@@@ @@@
                                        @@  @    @  @
                                       @@ @@@@  @@@@@@
                                      @@  @   @@@     @
                                     @@ @@@@ @@  @   @@@
                                    @@  @    @ @@@@ @@  @
                                   @@ @@@@  @@ @    @ @@@@
                                  @@  @   @@@  @@  @@ @   @
                                 @@ @@@@ @@  @@@ @@@  @@ @@@
                                @@  @    @ @@@   @  @@@  @  @
                               @@ @@@@  @@ @  @ @@@@@  @@@@@@@
                              @@  @   @@@  @@@@ @    @@@      @
                             @@ @@@@ @@  @@@    @@  @@  @    @@@
                            @@  @    @ @@@  @  @@ @@@ @@@@  @@  @
                           @@ @@@@  @@ @  @@@@@@  @   @   @@@ @@@@
                          @@  @   @@@  @@@@     @@@@ @@@ @@   @   @
...
```

The default output is "UnicodeAnsi" which uses ansi colors plus unicode block
elemets:

![rule 30 ascii](/../screenshots/2_3_150_1.png?raw=true "Rule 150 with UnicodeAnsi output")


Here is a 5 radix, 3 neighborhood rule with a random (@) starting config which pipes
its PNG output to img2sixel (using mintty terminal):

![5 3 random](/../screenshots/5_3_random.png?raw=true "Misc 5 radix rule")

## Building

Standard rust project, check out and run `cargo build`.

It is structured into a library + binary + tests, but only for ease of testing: no
thought has been put into how a CA API should be.

## limitations
## future directions

Arbitrary "code" specification. Code -> rule display (define inverse code mapping fn).

Reversable rule style?

Performance: I have barely looked at really tuning this. There is still some low
hanging fruit though.

Floating point cells? Perhaps these could work with code-specified rules.

Is it possible to get 8 distinct cells using unicode + ansi colors? I googled..

RGB colors get picked to be maximally differentiated, but what might look better
would be nice gradients. This should be some simple math I just don't want to
learn it just yet..

Sixel output. Rust libsixel bindings are fine I guess, but libsixel documentation
sucks. Meanwwhile do: `--output=PNG | img2sixel`
