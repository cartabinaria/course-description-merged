# Teaching page fixtures

Captured from public University of Bologna pages on 2026-10-21 for
[issue #130](https://github.com/cartabinaria/course-description-merged/issues/130).
These are reduced real HTML responses, not generated course content. No tests
request these URLs.

- `catalogue-it.html`: [the exact issue URL](https://www.unibo.it/it/studiare/insegnamenti-competenze-trasversali-moocs/insegnamenti?codiceMateria=08574&annoAccademico=2023&codiceCorso=8014&single=True&search=True).
- `catalogue-en.html`: [its English catalogue](https://www.unibo.it/en/study/course-units-transferable-skills-moocs/course-unit-catalogue?search=True&annoAccademico=2023&codiceCorso=8014&single=True&codiceMateria=08574), reached through the original page's language link.
- `operating-systems-it.html`: [the matching 08574 module](https://www.unibo.it/it/studiare/insegnamenti-competenze-trasversali-moocs/insegnamenti/insegnamento/2023/320660), linked from the Italian catalogue.
- `operating-systems-en.html`: [the English 08574 module](https://www.unibo.it/en/study/course-units-transferable-skills-moocs/course-unit-catalogue/course-unit/2023/320660), linked from the English catalogue.

All four requests returned HTTP 200. The issue URL is still a catalogue search,
not a teaching description: its integrated course contains both Operating
Systems (08574) and the laboratory (58348). Keep both links in the catalogue
fixtures so selecting the first result cannot pass as a correct resolver.

Reduction keeps the original opening `body` tag, English-language `li`, title
`div`, catalogue results `div` where present, and the English description `div`.
Each retained element comes directly from its response, preserving attributes,
text, line breaks and HTML entities; only trailing line whitespace is stripped.
The surrounding doctype and `html`/closing `body` wrappers are fixture
scaffolding. Unrelated navigation, styles, scripts and footer were removed.
Synthetic negative cases are explicit mutations inside the tests, not purported
captures.

SHA-256 of the complete UTF-8 response before reduction:

| Fixture | Original response SHA-256 |
| --- | --- |
| `catalogue-it.html` | `a034a5d8c6e9f287665d502c3e69bcbdf7b640fff150d4aba22ed02730e6c118` |
| `catalogue-en.html` | `145605694739893fe939c8cda530d61a10de8567dfb8eeffb0d3b94ecff92bbf` |
| `operating-systems-it.html` | `30e8311b1c228822cd934441f33c6439c75c3808c204e41625a4521e390e5954` |
| `operating-systems-en.html` | `d504473bbe8074a2e9698fc581afa28b8ba3b0217e1fd3d8e03f8755bbd80955` |
