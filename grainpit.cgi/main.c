#define _XOPEN_SOURCE 500

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "config.h"
#include "generated.h"

#define ARRAYSIZE(x) (sizeof(x)/sizeof(*(x)))

static unsigned char buf[262144];

int
main(int argc, char **argv)
{
	size_t bufpos, i;
	const char *ext, *path_info;

	path_info = getenv("PATH_INFO");
	if (path_info == NULL)
		path_info = "";
	if (path_info[0] == '/' && path_info[1] == 0)
		path_info = "";

	srandom(time(NULL));

	ext = strrchr(path_info, '.');
	if (ext == NULL)
		ext = "";
	if (path_info[0] != 0 && strcmp(ext, ".html") != 0) {
		buf[config_generate(buf, sizeof(buf) - 1, 512)] = 0;
		if (printf("Content-Type: text/plain; charset=UTF-8\n\n%s",
		    buf) < 0)
			return 1;
		return 0;
	}

	if (fputs(
	    "Content-Type: text/html; charset=UTF-8\n"
	    "\n"
	    "<h1>This is my website and it is Amazing!!</h1>\n",
	    stdout) == EOF)
		return 1;
	for (i = 0; i < 16; i++) {
		if (fputs("\n<a href='", stdout) == EOF)
			return 1;
		if ((random() % 100) < grainpit_extraurls_chance)
			if (fputs(grainpit_extraurls[random() %
			    ARRAYSIZE(grainpit_extraurls)], stdout) == EOF)
				return 1;
		bufpos = 0;
		bufpos += url_generate(
		    buf + bufpos, sizeof(buf) - bufpos - 3, 4);
		buf[bufpos++] = '/';
		bufpos += url_generate(
		    buf + bufpos, sizeof(buf) - bufpos - 2, 4);
		buf[bufpos++] = '/';
		bufpos += url_generate(
		    buf + bufpos, sizeof(buf) - bufpos - 1, 4);
		buf[bufpos++] = 0;
		if (printf("%s.html'>", buf) < 0)
			return 1;
		buf[url_name_generate(buf, sizeof(buf) - 1, 16)] = 0;
		if (printf("%s</a><br>", buf) < 0)
			return 1;
	}

	buf[html_generate(buf, sizeof(buf) - 1, 4096)] = 0;
	/* XXX modify href */
	if (fputs(buf, stdout) == EOF)
		return 1;

	return 0;
}
