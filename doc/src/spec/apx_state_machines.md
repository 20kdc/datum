# Appendix: State Machine Diagrams

## Tokenizer

* Begin: Clear token's working buffer and begin pushing characters to it (but not this character, unless it is otherwise consumed).
* Continue: Consume character. Push it to working buffer if that is enabled.
* Out: Output token of type from working buffer, clearing working buffer, and stop pushing. _Does not consume the character._

### Whitespace

In tokenizers that use a "reads from" model, implementing consuming whitespace as a dedicated function can be helpful.

```mermaid
stateDiagram-v2
	[*] --> Start
	Start --> Start: whitespace = Continue
	Start --> [*]: * = (to body...)
	Start --> EOF: eof = Continue
	Start --> LineComment: line-comment = Continue
	LineComment --> LineComment: * = Continue
	LineComment --> Start: newline = Continue
```

### Body

Assumes the whitespace at the beginning has been consumed.

When reading this diagram, EOF can be treated as an additional character class which is not part of any group.

```mermaid
stateDiagram-v2
	[*] --> Start
	Start --> [*]: alone = Out(ListStart/ListEnd), Continue
	Start --> NumericSign: sign = Begin, Continue
	NumericSign --> Numeric: potential-identifier = Continue
	NumericSign --> [*]: * = Out(ID)
	Start --> Numeric: digit = Begin, Continue
	Numeric --> Numeric: potential-identifier = Continue
	Numeric --> [*]: * = Out(Numeric)
	Start --> String: string = Begin
	String --> String: * = Continue
	String --> [*]: string = Out(String), Continue
	Start --> SpecialID: special-id = Begin
	SpecialID --> SpecialID: potential-identifier = Continue
	SpecialID --> [*]: * = Out(SpecialID)
```
