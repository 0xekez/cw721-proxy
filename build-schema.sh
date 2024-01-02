#!/bin/bash

FILES=`ls ./contracts`

for FILE in $FILES; do
    echo "cd ./contracts/$FILE; cargo schema"
    $(cd ./contracts/$FILE; cargo schema)
done