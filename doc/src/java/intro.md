# Java Implementation

This implementation was where Datum originally came from, in its original purpose as a replacement DSL for another, much worse DSL.

For 1.0.0, some APIs which can only be described as "less than stable" were stripped out.

It can be retrieved either from GitHub Packages (requires a read personal access token and annoying setup) or from my GitHub Pages (doesn't).

To use the GitHub Pages method, add this repository:

```xml
	<repositories>
		<repository>
			<id>kdc-pages</id>
			<name>20kdc GitHub Pages</name>
			<url>https://20kdc.github.io/maven</url>
		</repository>
	</repositories>
```

## Stability Guarantees

Changing versions around with Maven and the web of dependencies involved is messier than originally anticipated.

That in mind, this is bugfix-only unless there's a very, _very_ good reason.

The Java version requirement will never increase past Java 8.

Ideally it would have been lower, but tooling has settled on Java 8 as a sort of "last bastion" of legacy support, so you get toolchain issues if you try to go any lower.

## Serialization Framework?

None.

Pre-stabilization I reviewed some of the 'utility visitor' APIs that were being used in my downstream projects, and then immediately reworked them to deal with glaring issues.

They're still kind of a mess, and can't really be called stable.

As for reflection-based serialization, Java 8 doesn't really have the type system to express what would be necessary. It'd be a neverending, unbounded project with no clear end condition.

Considering my options, I've stripped the library down to the critical APIs.

## 'Push API' support?

None. The 'push API' only really worked at all in Rust, and frankly, even there it's a quick-fix for `no_std` and has ergonomics problems.
