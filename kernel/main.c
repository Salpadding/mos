#define VGA_START 0xc00b8000
#define VGA_WORDS 0x4000
#define VGA_LINES 25
#define VGA_COLS 80
#define LOADER_DATA (0x800 + 8)

// represent in binary
#define REPR_B __attribute__((packed))

typedef unsigned long u32_t;
typedef unsigned short u16_t;
typedef unsigned long long u64_t;

struct gdt_ptr
{
    u16_t gdt_bound; // gdt_size - 1
    u64_t *gdt_base
} REPR_B;

struct loader_data_st
{
    struct gdt_ptr* gdt_ptr_addr
} REPR_B;

// vga col  [0, 79]
static u32_t vga_col = 0;

int cls();
void next_line();
void put_char(char x);
void puts(const char *str);
void put_hex(u32_t u);

int main()
{
    puts("hello world!\n");
    struct loader_data_st *d = LOADER_DATA;

    puts("address of gdt_ptr:\n");
    put_hex(d->gdt_ptr_addr);
    put_char('\n');

    puts("bound of gdt:\n");
    put_hex(d->gdt_ptr_addr->gdt_bound);
    put_char('\n');


    puts("base of gdt:\n");
    put_hex(d->gdt_ptr_addr->gdt_base);
    put_char('\n');
    

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

void put_hex(u32_t x)
{
    static const char *chars = "0123456789abcdef";
    if (x == 0)
    {
        put_char('0');
        return;
    }

#define BUF_SIZE 32
    char buf[BUF_SIZE];
    int i = 0;

    while (x != 0)
    {
        u32_t y = x % 16;
        x = x / 16;
        buf[i] = chars[y];
        i++;
    }

    char buf2[BUF_SIZE];

    for (int j = 0; j < i; j++)
    {
        buf2[j] = buf[i - j - 1];
    }
    buf2[i] = '\0';
    puts("0x");
    puts(buf2);
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