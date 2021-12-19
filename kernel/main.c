#define VGA_START 0xc00b8000 
#define VGA_WORDS  0x4000
#define VGA_LINES  25
#define VGA_COLS   80

typedef unsigned long u32_t;
typedef unsigned short  u16_t;

// vga line [0, 24]
static u32_t vga_line = VGA_LINES - 1;
// vga col  [0, 79]
static u32_t vga_col = 0;

int cls();
void next_line();
void puts(const char* str);

int main() {
    cls();
    puts("hello world!\n");
    int a = 0;
    int b = 0;
    while(1) {
        a += 1;
        b = a * a;
    }
}

// clear vga buffer
int cls() {
   u16_t * vga = VGA_START;

   for(u32_t i = 0; i < VGA_WORDS; i++) {
       vga[i] = 0x0f20;
   }

   return 0;
}

void puts(const char* str) {
    u16_t * vga = VGA_START;
    u32_t i = 0;
    while(1) {
        char c = str[i];
        if (c == '\0') {
            break;
        }

        if (c == '\n') {
            next_line();
            i++;
            continue;
        }

        u16_t cx = c;
        cx = cx | 0x0f00;
        vga[(VGA_LINES - 1) * VGA_COLS + vga_col] = cx;

        if (vga_col == VGA_COLS - 1) {
            vga_col = 0;
            next_line();
        } else {
            vga_col++;
        }

        i++;
    }
}

void next_line() {
    u16_t* vga = VGA_START;
    // scroll screen by a line
    for(u32_t i = 0; i < VGA_LINES -1; i++) {
        for(u32_t j = 0; j < VGA_COLS; j++) {
            vga[i * VGA_COLS + j] = vga[(i+1) * VGA_COLS + j];
        }
    }

    for(u32_t i = 0; i < VGA_COLS; i++) {
        vga[((VGA_LINES - 1) * VGA_COLS) + i] = 0x0f20;
    }
    vga_col = 0;
} 