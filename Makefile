export PKG_CONFIG_PATH := /opt/homebrew/opt/cups/lib/pkgconfig

# export LDFLAGS="-L/usr/lib"
# export CPPFLAGS="-I/usr/include"

CC = gcc
CFLAGS = -I./include $(shell pkg-config --cflags cups) 
LDFLAGS = -L/usr/lib -lcups -lcupsimage
SRC = rastertolabel.c
OBJ = $(SRC:.c=.o)
TARGET = rastertolabel
DESTDIR = /Library/Printers/VevorPrinter300/Filter

all: $(TARGET) install start

$(TARGET): $(OBJ)
	$(CC) -o $@ $^ $(LDFLAGS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

install: $(TARGET)
	sudo install -m 755 $(TARGET) $(DESTDIR)

start: $(TARGET)
	sudo ./$(TARGET)

clean:
	rm -f $(OBJ) $(TARGET)

.PHONY: all clean 