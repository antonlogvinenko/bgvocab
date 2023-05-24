cargo build
../target/debug/vocab-bin --quiz --batch-size=50 --batch-number=16 --double --pdf
rm ~/Dropbox/Bulgarian/words/*.pdf
mv *.pdf ~/Dropbox/Bulgarian/words/
