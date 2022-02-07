# musescore.com rip

>Download any score from the community sheet publishing website musescore.com.
Enter the scores URL as the argument. Based on [headless-chrome](https://github.com/atroche/rust-headless-chrome).

If it doesn't work, inspect element on any sheet page and add the class name of page div as an argument --div-class. You will see that this class is the same for all the pages.

in this case the correct class is "vAVs3"

```html
<div class="vAVs3"> <!-- Add this as --div-class argument -->
    <img class="_2zZ8u"> <!-- This is the .svg file of the page -->
    <div class="_2veuP">
        <div class="_1gf3p _1t3VX"></div>
    </div>
    <div class="_1gf3p _1t3VX"></div>
    <div class="_1gf3p _1t3VX"></div>
</div>
```