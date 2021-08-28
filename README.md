# Text to Polly SSML #

A Library to turn Text into Valid "Polly SSML". Note I say Polly SSML Here, since the goal for this is
to be eventually sent to AWS Polly. AWS Polly does not implement the full SSML v1.1 Spec. It implements
a subset of it, and as such that is the subset we support. E.g. if you can't do it in polly, you can't do it
here.

## Why do I need this? ##

Let's be honest. You want polly somewhere accessible at all times. Justins Voice can brighten any situation,
and that's awesome. However you know what isn't awesome? Spending tons of time crafting out XML. That's not super
cool. So you need this in order to be able to type in text.

For example I've deployed this as part of my slackbot, so I can type hilarious SSML messages right in chat. Without
having to type XML.

The general format is:

```text
${keyname|param_key=param_value} some text here ${/keyname}
```

So for example if I wanted to use the `prosody` tag as they defined: [HERE](http://docs.aws.amazon.com/polly/latest/dg/supported-ssml.html)
I'd type a message like:

```text
${prosody|volume=+14dB|pitch=+200%|rate=x-fast}coffee coffee coffee${/prosody}
```

Which would generate SSML That looked like:

```xml
<?xml version="1.0"?><speak xml:lang="en-US" onlangfailure="processorchoice" xmlns="http://www.w3.org/2001/10/synthesis" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <prosody volume="+14dB" pitch="+200%" rate="x-fast">
    coffee coffeecoffee
  </prosody>
</speak>
```

Of course the first one is more terse, but not by much. However when you're going several keys deep, and many params it can be.

## Usage ##

Simply import the library as a crate, and call parse_string:

```rust
extern crate text_to_polly_ssml;

fn main() {
  let result = text_to_polly_ssml::parse_string("my string".to_owned());
  assert!(result.is_ok());
  let ssml = result.unwrap();
}
```

You can alternative call `parse_str`:

```rust
extern crate text_to_polly_ssml;

fn main() {
  let result = text_to_polly_ssml::parse_str("my string");
  assert!(result.is_ok());
  let ssml = result.unwrap();
}
```


## License ##

This library is licensed under MIT.