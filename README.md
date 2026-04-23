# 4d Factory Game

## Space Axes

X - forwards/backwards<br>
Y - up/down<br>
Z - right/left<br>
W - ana/kata<br>

## Controls

W/S - move forwards/backwards<br>
A/D - move left/right<br>
E/Q - move up/down<br>
R/F - move ana/kata<br>

G - toggle "ground view"<br>

T - toggle "screen door view"<br>
Y/H - increase/decrease the percentage of offset of the "screen door view" rays<br>

Left Click - Place a hypersphere with a random color (or overwrite any hypersphere that is already there)<br>
Right Click - Delete a hypersphere<br>

While shift not pressed:<br>
Up/Down arrow keys - rotate xy<br>
Left/Right arrow keys - rotate xz<br>

While shift pressed:<br>
Up/Down arrow keys - rotate xw<br>
Left/Right arrow keys - rotate zw<br>

## Ground View

The camera is given a 90 degree yw rotation to basically replace the y axis with the w axis so that you can see everything horizontally to you.<br>
Make sure you are on the same y level as everything that you want to see in ground view<br>

## Screen Door View

The screen is broken up into a grid containing 9 pixels in each cell. Each pixel going from left to right, top to bottom, is a ray that is offset a tiny bit in the ana/kata axis depending on the pixel.<br>
The top-left pixel is the most kata ray, the center pixel has no offset, and the bottom-right pixel is the most ana ray.<br>
It effectively lets you see 9 different 3d slices of the world at the same time.
