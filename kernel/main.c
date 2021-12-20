#define VGA_START 0xc00b8000
#define VGA_WORDS 0x4000
#define VGA_LINES 25
#define VGA_COLS 80
#define ASM_API (0x800 + 1024)

typedef unsigned long u32_t;
typedef unsigned short u16_t;

// vga col  [0, 79]
static u32_t vga_col = 0;

typedef u32_t (*asm_api)(u32_t x);

int cls();
void next_line();
void put_char(char x);
void puts(const char *str);

int main()
{
    asm_api e = ASM_API;
    u32_t rt = e('a');
    cls();
    put_char('a');
    put_char('\n');
    puts("hello world!\n");
    int a = 0;
    int b = 0;
    while (1)
    {
        a += 1;
        b = a * a;
    }
}

// clear vga buffer
int cls()
{
    u16_t *vga = VGA_START;

    for (u32_t i = 0; i < VGA_WORDS; i++)
    {
        vga[i] = 0x0f20;
    }

    return 0;
}

void put_char(char c)
{
    u16_t *vga = VGA_START;
    if (c == '\0')
    {
        return;
    }

    if (c == '\n')
    {
        next_line();
        return;
    }

    u16_t cx = c;
    cx = cx | 0x0f00;
    vga[(VGA_LINES - 1) * VGA_COLS + vga_col] = cx;

    if (vga_col == VGA_COLS - 1)
    {
        vga_col = 0;
        next_line();
    }
    else
    {
        vga_col++;
    }
}

void puts(const char *str)
{
    u16_t *vga = VGA_START;
    u32_t i = 0;
    while (1)
    {
        char c = str[i];
        if (c == '\0')
            break;
        put_char(c);
        i++;
    }
}

void next_line()
{
    u16_t *vga = VGA_START;
    // scroll screen by a line
    for (u32_t i = 0; i < VGA_LINES - 1; i++)
    {
        for (u32_t j = 0; j < VGA_COLS; j++)
        {
            vga[i * VGA_COLS + j] = vga[(i + 1) * VGA_COLS + j];
        }
    }

    for (u32_t i = 0; i < VGA_COLS; i++)
    {
        vga[((VGA_LINES - 1) * VGA_COLS) + i] = 0x0f20;
    }
    vga_col = 0;
}