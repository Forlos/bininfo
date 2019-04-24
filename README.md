<a href="http://spacemacs.org"><img src="https://cdn.rawgit.com/syl20bnr/spacemacs/442d025779da2f62fc86c2082703697714db6514/assets/spacemacs-badge.svg" alt="Made with Spacemacs"></a><br>
# Bininfo 
Get information about various binary file formats 

Warning: Error handling is very basic and hasn't been tested.

![png_example1](Media/Screenshot1.png)
![definitely_not_stolen](Media/Screenshot2.png)

![More screenshots](Media/README.md)

## Supported formats
- [x] BMP
- [x] PNG 1.2, PNGEXT 1.2
- [x] GIF
- [ ] JPG
- [ ] PDF
- [x] ELF
- [x] PE(No exports, symbols)
- [x] JAVA CLASS(11)
- [x] MACH-O(No imports, exports)
- [x] LUA(5.1)
- [ ] ZIP

## Resources

### Bmp
- https://entropymine.com/jason/bmpsuite/

### Elf
- https://github.com/m4b/goblin
- http://www.skyfree.org/linux/references/ELF_Format.pdf

### Gif
- https://www.w3.org/Graphics/GIF/spec-gif89a.txt

### JavaClass
- https://docs.oracle.com/javase/specs/jvms/se11/html/jvms-4.html

### Lua
- http://files.catwell.info/misc/mirror/lua-5.2-bytecode-vm-dirk-laurie/lua52vm.html
- http://luaforge.net/docman/83/98/ANoFrillsIntroToLua51VMInstructions.pdf

### Mach-O
- https://opensource.apple.com/source/xnu/xnu-4903.221.2/EXTERNAL_HEADERS/mach-o/loader.h.auto.html
- https://opensource.apple.com/source/xnu/xnu-4903.221.2/osfmk/mach/

### Pe
- https://docs.microsoft.com/en-us/windows/desktop/Debug/pe-format
- http://www.win32assembly.programminghorizon.com/pe-tut1.html

### Png
- http://www.libpng.org/pub/png/pngsuite.html
- http://www.libpng.org/pub/png/png-sitemap.html#images

### Zip
- https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT

### Other
- https://github.com/corkami
- https://en.wikipedia.org/wiki/List_of_file_signatures
- http://fileformats.archiveteam.org/wiki/Main_Page
