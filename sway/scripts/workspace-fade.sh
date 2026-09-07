#!/bin/bash

# Fade overlay
swaymsg "output * dpms off"
sleep 0.05
swaymsg "workspace number $1"
swaymsg "output * dpms on"
