# bgvocab
Tool for studying new words

https://user-images.githubusercontent.com/166523/235342915-25d70f74-d17e-45dd-8084-931d316ec82e.mov

Select **page size** and **page number** of vocabulary to display:
```
cargo run -- --batch-size=10 --batch-number=0
```

Select **quiz mode**: additional **Enter** or **Backspace** is required to see the translation:
```
cargo run -- --batch-size=10 --batch-number=0 --quiz
```

Select **bg-en vocabulary** instead of **bg-ru**:
```
cargo run -- --batch-size=10 --batch-number=0 --en
```

# Installing cargo and rust
  Just follow the instructions https://www.rust-lang.org/tools/install
