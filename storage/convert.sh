#!/bin/bash

for f in ./*.png; do
    convert "$f" -background black -alpha remove "$f.tga"
done

