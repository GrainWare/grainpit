/* I'm sorry for you if you have to read this. */

#define _POSIX_C_SOURCE 200809L

#include <ctype.h>
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define hcf() do { \
	fprintf(stderr, "big bad error at %s:%d, errno: %s\n", \
	    __FILE__, __LINE__, strerror(errno)); \
	unlink("generated.h"); \
	exit(1); \
} while(0)

#define TOKEN_MAX_LENGTH 4096
/* needs to be a power of 2 */
#define TOKEN_MAX_COUNT 4096
#define NEXT_TOK_ODDS_MAX_LEN 262144
#define MAP_MAX_LEN 131072
#define MAP_MAX_MASK (MAP_MAX_LEN - 1)
#define TOKEN_CONTENTS_MAX_LENGTH 1048576
#define CONTEXT_WINDOW_LENGTH 5

struct tokodds {
	uint32_t context[CONTEXT_WINDOW_LENGTH];
	uint16_t odds[TOKEN_MAX_COUNT];
};
struct finaltokodds {
	uint32_t context[CONTEXT_WINDOW_LENGTH];
	uint32_t totalodds;
	uint32_t nexttokoddsoff;
	uint32_t nexttokoddslen;
};
struct nexttokoddsent {
	uint32_t tok;
	uint32_t odds;
};

static FILE *out;
static struct tokodds *oddsmap;
static struct finaltokodds *finaloddsmap;

/* Murmur3_32 hash */
static uint32_t
ctxhash(const uint32_t *in, size_t len, uint32_t seed)
{
	size_t i;
	uint32_t h, k;

	h = seed;
	i = 0;
	for (i = 0; i < CONTEXT_WINDOW_LENGTH; i++) {
		k = i < len ? in[i] : 0xFFFFFFFF;
		k *= 0xcc9e2d51;
		k = (k << 15) | (k >> 17); /* ROL 15 */
		k *= 0x1b873593;
		h ^= k;
		h = (h << 13) | (h >> 19); /* ROL 13 */
		h *= 5;
		h += 0xe6546b64;
	}
	h ^= len;
	h ^= h >> 16;
	h *= 0x85ebca6b;
	h ^= h >> 13;
	h *= 0xc2b2ae35;
	h ^= h >> 16;

	return h;
}

static int
contexteql(const uint32_t *restrict context, size_t len,
    const struct tokodds *restrict tokodds)
{
	size_t i;

	for (i = 0; i < len; i++)
		if (tokodds->context[i] != context[i])
			return 0;
	if (i < CONTEXT_WINDOW_LENGTH && tokodds->context[i] != 0xFFFFFFFF)
		return 0;
	return 1;
}

static void
train(const char *name, uint32_t seed)
{
	FILE *f;
	size_t i, j, k, toklen, tokcount;
	int c;
	static struct nexttokoddsent nexttokodds[NEXT_TOK_ODDS_MAX_LEN];
	static unsigned char tokcont[TOKEN_CONTENTS_MAX_LENGTH];
	/* + 1 for NUL */
	static unsigned char tok[TOKEN_MAX_LENGTH + 1];
	char path[PATH_MAX];
	static uint32_t tokoffs[TOKEN_MAX_COUNT];
	uint32_t context[CONTEXT_WINDOW_LENGTH], duplicates, nexttokoddslen,
	    nexttokoddsoff, oddsmaplen, oddsmapmask, tokcontlen, tokind,
	    trunchash;

	memset(oddsmap, 0xFF, sizeof(*oddsmap) * MAP_MAX_LEN);
	memset(finaloddsmap, 0xFF, sizeof(*finaloddsmap) * MAP_MAX_LEN);
	memset(context, 0xFF, sizeof(context));
	nexttokoddsoff = 0;
	oddsmaplen = 0;
	tokcount = 0;
	tokcontlen = 0;

	/* yes, there is no length check. no i don't care */
	sprintf(path, "../grainpit/data/%s.txt", name);
	f = fopen(path, "rb");
	if (f == NULL)
		hcf();

	for (;;) {
		c = fgetc(f);
		if (c == EOF) {
			if (feof(f))
				break;
			hcf();
		}
		if (isalnum(c)) {
			tok[0] = c;
			i = 1;
			for (;;) {
				if (i >= TOKEN_MAX_LENGTH)
					hcf();
				c = fgetc(f);
				if (c == EOF) {
					if (feof(f))
						break;
					hcf();
				}
				if (!isalnum(c)) {
					if (ungetc(c, f) == EOF)
						hcf();
					break;
				}
				tok[i] = c;
				i++;
			}
		} else if (isspace(c)) {
			tok[0] = c;
			i = 1;
			for (;;) {
				if (i >= TOKEN_MAX_LENGTH)
					hcf();
				c = fgetc(f);
				if (c == EOF) {
					if (feof(f))
						break;
					hcf();
				}
				if (!isspace(c)) {
					if (ungetc(c, f) == EOF)
						hcf();
					break;
				}
				tok[i] = c;
				i++;
			}
		} else if (c == '<') {
			tok[0] = c;
			i = 1;
			for (;;) {
				if (i >= TOKEN_MAX_LENGTH)
					hcf();
				c = fgetc(f);
				if (c == EOF) {
					if (feof(f))
						break;
					hcf();
				}
				tok[i] = c;
				i++;
				if (c == '>')
					break;
			}
		} else {
			if ((c & 0x80) == 0x00)
				toklen = 1;
			else if ((c & 0xC0) == 0x80)
				hcf();
			else if ((c & 0xE0) == 0xC0)
				toklen = 2;
			else if ((c & 0xF0) == 0xE0)
				toklen = 3;
			else if ((c & 0xF8) == 0xF0)
				toklen = 4;
			else
				hcf();
			tok[0] = c;
			i = 1;
			while (i < toklen) {
				c = fgetc(f);
				if (c == EOF) {
					if (feof(f)) {
						/* different msg for EOF */
						errno = 0;
						hcf();
					}
					hcf();
				}
				tok[i] = c;
				i++;
			}
		}

		toklen = i;
		tok[toklen] = 0;

		for (i = 0; i < tokcount; i++) {
			if (memcmp(tok, tokcont + tokoffs[i], toklen + 1) == 0)
				break;
		}
		if (i == tokcount) {
			if (tokcount == TOKEN_MAX_COUNT)
				hcf();
			if ((tokcontlen + toklen + 1) >
			    TOKEN_CONTENTS_MAX_LENGTH)
				hcf();
			memcpy(tokcont + tokcontlen, tok, toklen + 1);
			tokoffs[i] = tokcontlen;
			tokcontlen += toklen + 1;
			tokcount++;
		}

		tokind = i;

		for (i = 1; i <= CONTEXT_WINDOW_LENGTH; i++) {
			if (context[i - 1] == 0xFFFFFFFF) break;
			trunchash = ctxhash(context, i, seed);
			trunchash &= MAP_MAX_MASK;
			j = trunchash;
			while (!contexteql(context, i, &oddsmap[j])) {
				if (oddsmap[j].context[0] != 0xFFFFFFFF) {
					j++;
					j &= MAP_MAX_MASK;
					if (j == trunchash)
						hcf();
					continue;
				}
				memcpy(oddsmap[j].context, context,
				    sizeof(*oddsmap[j].context) * i);
				memset(oddsmap[j].context + i, 0xFF,
				    sizeof(*oddsmap[j].context) *
				    (CONTEXT_WINDOW_LENGTH - i));
				memset(oddsmap[j].odds, 0x00,
				    sizeof(oddsmap[j].odds));
				oddsmaplen++;
				break;
			}
			oddsmap[j].odds[tokind]++;
		}
		memmove(context + 1, context,
		    sizeof(*context) * (CONTEXT_WINDOW_LENGTH - 1));
		context[0] = tokind;
	}

	/* oddsmaplen = (oddsmaplen * 1.5) rounded to next higher power of 2 */
	oddsmaplen += oddsmaplen >> 1;
	oddsmaplen--;
	oddsmaplen |= oddsmaplen >> 1;
	oddsmaplen |= oddsmaplen >> 2;
	oddsmaplen |= oddsmaplen >> 4;
	oddsmaplen |= oddsmaplen >> 8;
	oddsmaplen |= oddsmaplen >> 16;
	oddsmapmask = oddsmaplen;
	oddsmaplen++;

	if (oddsmaplen > MAP_MAX_LEN) {
		fprintf(stderr, "%s odds map truncated\n", name);
		oddsmapmask = MAP_MAX_MASK;
		oddsmaplen = MAP_MAX_LEN;
	}

	duplicates = 0;
	for (i = 0; i < MAP_MAX_LEN; i++) {
		if (oddsmap[i].context[0] == 0xFFFFFFFF)
			continue;

		trunchash = ctxhash(oddsmap[i].context, CONTEXT_WINDOW_LENGTH,
		    seed);
		trunchash &= oddsmapmask;
		j = trunchash;


		if (finaloddsmap[j].context[0] != 0xFFFFFFFF)
			duplicates++;

		while (finaloddsmap[j].context[0] != 0xFFFFFFFF) {
			j++;
			j &= oddsmapmask;
		}

		memcpy(&finaloddsmap[j].context, &oddsmap[i].context,
		    sizeof(finaloddsmap[j].context));
		finaloddsmap[j].totalodds = 0;

		nexttokoddslen = 0;
		for (k = 0; k < tokcount; k++) {
			finaloddsmap[j].totalodds += oddsmap[i].odds[k];
			if (oddsmap[i].odds[k] != 0)
				nexttokoddslen++;
		}

		if ((nexttokoddsoff + nexttokoddslen) > NEXT_TOK_ODDS_MAX_LEN)
			hcf();
		finaloddsmap[j].nexttokoddsoff = nexttokoddsoff;
		finaloddsmap[j].nexttokoddslen = nexttokoddslen;

		for (k = 0; k < tokcount; k++) {
			if (oddsmap[i].odds[k] == 0)
				continue;
			nexttokodds[nexttokoddsoff].tok = k;
			nexttokodds[nexttokoddsoff].odds = oddsmap[i].odds[k];
			nexttokoddsoff++;
		}
	}

	if (duplicates != 0)
		fprintf(stderr, "%s map has %" PRIu32 " duplicates\n",
		    name, duplicates);

	if (fprintf(out,
	    "#define %s_TOKEN_COUNT %zu\n"
	    "#define %s_MAP_MASK 0x%.8" PRIx32 "\n"
	    "#define %s_HASH_SEED 0x%.8" PRIx32 "\n"
	    "struct %s_ctx_odds {\n"
	    "\tuint32_t ctx[CONTEXT_WINDOW_LENGTH];\n"
	    "\tuint32_t total_odds;\n"
	    "\tuint32_t tok_odds_off;\n"
	    "\tuint32_t tok_odds_len;\n"
	    "};\n"
	    "struct %s_tok_odds {\n"
	    "\tuint32_t tok;\n"
	    "\tuint32_t odds;\n"
	    "};\n"
	    "static struct %s_ctx_odds %s_ctx_odds_map[] = {\n",
	    name, tokcount, name, oddsmapmask, name, seed,
	    name, name, name, name) < 0)
		hcf();

	for (i = 0; i < oddsmaplen; i++) {
		if (fputs("\t{ { ", out) == EOF)
			hcf();
		for (j = 0; j < CONTEXT_WINDOW_LENGTH; j++)
			if (fprintf(out, "0x%.8" PRIx32 ", ",
			    finaloddsmap[i].context[j]) < 0)
				hcf();
		if (fprintf(out,
		    "}, %10" PRIu32 ", %10" PRIu32 ", 0x%.8" PRIx32 " },\n",
		    finaloddsmap[i].totalodds, finaloddsmap[i].nexttokoddsoff,
		    finaloddsmap[i].nexttokoddslen) < 0)
			hcf();
	}

	if (fprintf(out,
	    "};\n"
	    "static struct %s_tok_odds %s_tok_odds_list[] = {\n",
	    name, name) < 0)
		hcf();

	for (i = 0; i < nexttokoddsoff; i++)
		if (fprintf(out,
		    "\t{ 0x%.8" PRIx32 ", 0x%.8" PRIx32 " },\n",
		    nexttokodds[i].tok, nexttokodds[i].odds) < 0)
			hcf();

	if (fprintf(out,
	    "};\n"
	    "static uint32_t %s_offs[] = { ",
	    name) < 0)
		hcf();

	for (i = 0; i < tokcount; i++)
		if (fprintf(out, "%" PRIu32 ", ", tokoffs[i]) < 0)
			hcf();

	if (fprintf(out,
	    "};\n"
	    "static unsigned char %s_cont[] = { ",
	    name) < 0)
		hcf();

	for (i = 0; i < tokcontlen; i++)
		if (fprintf(out, "0x%.2hhx, ", tokcont[i]) < 0)
			hcf();

	if (fprintf(out,
	    "};\n"
	    "\n"
	    "static size_t\n"
	    "%s_generate(unsigned char *out, size_t outlen, size_t tokens)\n"
	    "{\n"
	    "\tsize_t ctxlen, i, j;\n"
	    "\tconst unsigned char *tokname, *startout;\n"
	    "\tuint32_t ctx[CONTEXT_WINDOW_LENGTH], rngval, tokind, "
	    "trunchash;\n"
	    "\n"
	    "\tstartout = out;\n"
	    "\n"
	    "\ttokind = random() %% %s_TOKEN_COUNT;\n"
	    "\tfor (i = 0; i < CONTEXT_WINDOW_LENGTH; i++)\n"
	    "\t\tctx[i] = 0xFFFFFFFF;\n"
	    "\n"
	    "\tfor (;;) {\n"
	    "\t\ttokname = %s_cont + %s_offs[tokind];\n"
	    "\t\twhile (*tokname != 0x00) {\n"
	    "\t\t\t*out = *tokname;\n"
	    "\t\t\tout++;\n"
	    "\t\t\toutlen--;\n"
	    "\t\t\tif (outlen == 0)\n"
	    "\t\t\t\treturn out - startout;\n"
	    "\t\t\ttokname++;\n"
	    "\t\t}\n"
	    "\n"
	    "\t\ttokens--;\n"
	    "\t\tif (tokens == 0)\n"
	    "\t\t\treturn out - startout;\n"
	    "\n"
	    "\t\tmemmove(ctx + 1, ctx,\n"
	    "\t\t    sizeof(*ctx) * (CONTEXT_WINDOW_LENGTH - 1));\n"
	    "\t\tctx[0] = tokind;\n"
	    "\n"
	    "\t\tfor (ctxlen = CONTEXT_WINDOW_LENGTH; ctxlen > 0; ctxlen--) {\n"
	    "\t\t\ttrunchash = markov_ctxhash(ctx, ctxlen, %s_HASH_SEED);\n"
	    "\t\t\ttrunchash &= %s_MAP_MASK;\n"
	    "\t\t\ti = trunchash;\n"
	    "\t\t\tfor (;;) {\n"
	    "\t\t\t\tfor (j = 0; j < CONTEXT_WINDOW_LENGTH; j++)\n"
	    "\t\t\t\t\tif (ctx[j] != %s_ctx_odds_map[i].ctx[j]) {\n"
	    "\t\t\t\t\t\ti++;\n"
	    "\t\t\t\t\t\ti &= %s_MAP_MASK;\n"
	    "\t\t\t\t\t\tif (i == trunchash)\n"
	    "\t\t\t\t\t\t\tgoto nextctxwindow;\n"
	    "\t\t\t\t\t\tgoto nextctxodds;\n"
	    "\t\t\t\t\t}\n"
	    "\t\t\t\tbreak;\n"
	    "nextctxodds:\n"
	    "\t\t\t\tcontinue;\n"
	    "\t\t\t}\n"
	    "\n"
	    "\t\t\trngval = random() %% %s_ctx_odds_map[i].total_odds;\n"
	    "\t\t\tfor (j = %s_ctx_odds_map[i].tok_odds_off;\n"
	    "\t\t\t    j < (%s_ctx_odds_map[i].tok_odds_off +\n"
	    "\t\t\t    %s_ctx_odds_map[i].tok_odds_len);\n"
	    "\t\t\t    j++) {\n"
	    "\t\t\t\tif (%s_tok_odds_list[j].odds > rngval)\n"
	    "\t\t\t\t\tbreak;\n"
	    "\t\t\t\trngval -= %s_tok_odds_list[j].odds;\n"
	    "\t\t\t}\n"
	    "\n"
	    "\t\t\ttokind = %s_tok_odds_list[j].tok;\n"
	    "\t\t\tgoto nexttoken;\n"
	    "nextctxwindow:\n"
	    "\t\t\tcontinue;\n"
	    "\t\t}\n"
	    "\t\t/* dead end, stop here */\n"
	    "\t\tbreak;\n"
	    "nexttoken:\n"
	    "\t\tcontinue;\n"
	    "\t}\n"
	    "\n"
	    "\treturn out - startout;\n"
	    "}\n",
	    name, name, name, name, name, name, name, name, name, name, name,
	    name, name, name, name) < 0)
		    hcf();

	fclose(f);
}

int
main(void)
{
	oddsmap = malloc(MAP_MAX_LEN * sizeof(*oddsmap));
	if (oddsmap == NULL)
		hcf();
	finaloddsmap = malloc(MAP_MAX_LEN * sizeof(*finaloddsmap));
	if (finaloddsmap == NULL)
		hcf();

	out = fopen("generated.h", "w");
	if (out == NULL)
		hcf();

	if (fprintf(out,
	    "/* Machine generated code, see generate.c */\n"
	    "\n"
	    "#include <stdint.h>\n"
	    "#include <stdlib.h>\n"
	    "#include <string.h>\n"
	    "\n"
	    "#define CONTEXT_WINDOW_LENGTH %d\n"
	    "\n"
	    "/* Murmur3_32 hash */\n"
	    "static uint32_t\n"
	    "markov_ctxhash(const uint32_t *in, size_t len, uint32_t seed)\n"
	    "{\n"
	    "\tsize_t i;\n"
	    "\tuint32_t h, k;\n"
	    "\n"
	    "\th = seed;\n"
	    "\ti = 0;\n"
	    "\tfor (i = 0; i < CONTEXT_WINDOW_LENGTH; i++) {\n"
	    "\t\tk = i < len ? in[i] : 0xFFFFFFFF;\n"
	    "\t\tk *= 0xcc9e2d51;\n"
	    "\t\tk = (k << 15) | (k >> 17); /* ROL 15 */\n"
	    "\t\tk *= 0x1b873593;\n"
	    "\t\th ^= k;\n"
	    "\t\th = (h << 13) | (h >> 19); /* ROL 13 */\n"
	    "\t\th *= 5;\n"
	    "\t\th += 0xe6546b64;\n"
	    "\t}\n"
	    "\th ^= len;\n"
	    "\th ^= h >> 16;\n"
	    "\th *= 0x85ebca6b;\n"
	    "\th ^= h >> 13;\n"
	    "\th *= 0xc2b2ae35;\n"
	    "\th ^= h >> 16;\n"
	    "\n"
	    "\treturn h;\n"
	    "}\n"
	    "\n",
	    CONTEXT_WINDOW_LENGTH) < 0)
		hcf();

	train("html", 0x00000001);
	train("url_name", 0x00000005);
	train("config", 0x00000004);
	train("url", 0x00000010);

	return 0;
}
