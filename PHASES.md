1. Copy the bare minimum from cloud filter.
    Not using fancy windows IO, just stdlib stuff.
    1. literally just the file copying, no other add-ons.
       establish that it works. No updates.
       Even just using a blocking API.
    2. Also with file watching / updates. (can this be blocking?)
    3. Asyncify.
    4. Make a little more idiomatic.
    5. Idk I had something here from before but I forgot.
    

# side notes
- what kind of fuzzing / address sanitization tools are there to make sure our PCWSTR fuckery is okay?
- bruh conversion should be lazy, just offer the slice
- how can I make `FromVoid` allow just using it as a reference, but enforce it as static if it won't be?
    - does it need to be _ToStructure_ for the sake of lifetime specs?

- I wish there was `anyhow` but with type-safe, same errors

- for a trampoline, offer a macro to generate the callback functions.
  - I _could_ make it work with multiple implementations too, but that seems highly unnecessary.
    - do I even want this to be a generic library?

- seriously, who is supposed to be in charge of the memory in the callback info / params?