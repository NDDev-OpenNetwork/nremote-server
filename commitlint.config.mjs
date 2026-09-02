// Conventional Commits, with one narrow exemption to the line-length rules.
//
// `body-max-line-length` and `footer-max-line-length` exist so that `git log`
// stays readable in a narrow terminal, and for prose that is the right rule.
// It is not achievable for a line that is long because of a URL. Dependabot
// writes several such lines into every dependency bump:
//
//   - [Commits](https://github.com/<org>/<repo>/compare/<40 hex>...<40 hex>)
//   | [rustls-platform-verifier](https://github.com/rustls/rustls-platform-verifier) | `0.6.2` | `0.7.0` |
//
// 144 and 102 characters, and neither has a wrap point that would help: the
// first is one link, the second is a table row. With the stock rule every such
// pull request fails commitlint, is blocked from merging, and the dependency
// never lands - which is how a security update stops arriving.
//
// So the limit still applies, and it is measured over what the author could
// have wrapped. Each URL counts as one character; if the line is still too
// long after that, nobody wrapped it and the rule fires. A 119-character line
// of ordinary words fails, and so does a 130-character line of prose that
// happens to contain a short link. A line whose length is a link is exempt.
//
// A single non-URL token longer than the limit - a hash, a base64 blob, a very
// deep path - is exempt on the same reasoning.
//
// The controls are in `scripts/check_commitlint_config.mjs`, which CI runs.

const LIMIT = 100;

// Stops at whitespace and at a closing paren, so the `)` and `.` that end a
// markdown link are counted as the text they are.
const URL_PATTERN = /https?:\/\/[^\s)]+/g;

const unwrappable = (line) => {
  if (line.split(/\s+/).some((token) => token.length > LIMIT)) {
    return true;
  }
  return line.replace(URL_PATTERN, '').length <= LIMIT;
};

const offendingLines = (text) =>
  (text ?? '')
    .split('\n')
    .filter((line) => line.length > LIMIT)
    .filter((line) => !unwrappable(line));

const wrappableLineLength = (section) => (parsed) => {
  const offenders = offendingLines(parsed[section]);
  const detail = offenders
    .map((line) => `  ${line.length} chars: ${line.slice(0, 60)}...`)
    .join('\n');
  return [
    offenders.length === 0,
    `${section} lines must not be longer than ${LIMIT} characters. A line is ` +
      `exempt only when its length comes from a URL or from one very long ` +
      `token:\n${detail}`,
  ];
};

export const testable = { LIMIT, unwrappable, offendingLines };

export default {
  extends: ['@commitlint/config-conventional'],
  plugins: [
    {
      rules: {
        'body-max-line-length-wrappable': wrappableLineLength('body'),
        'footer-max-line-length-wrappable': wrappableLineLength('footer'),
      },
    },
  ],
  rules: {
    // Replaced, not relaxed: the two rules below enforce the same limit.
    'body-max-line-length': [0, 'always', LIMIT],
    'footer-max-line-length': [0, 'always', LIMIT],
    'body-max-line-length-wrappable': [2, 'always'],
    'footer-max-line-length-wrappable': [2, 'always'],
  },
};
