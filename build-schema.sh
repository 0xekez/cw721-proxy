#!/bin/bash

FILES=`ls ./contracts`

for FILE in $FILES; do
    $(cd ./contracts/$FILE; cargo schema)
done