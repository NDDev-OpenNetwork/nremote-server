// Conventional Commits, with one narrow exemption to the line-length rules.
//
// `body-max-line-length` and `footer-max-line-length` exist so that `git log`
// stays readable in a narrow terminal, and for prose that is the right rule.
// It is not achievable for a line whose content is a single URL. Dependabot
// writes one on every dependency it moves by git ref:
//
//   - [Commits](https://github.com/<org>/<repo>/compare/<40 hex>...<40 hex>)
//
// That line is 144 characters and contains no wrap point. With the stock rule
// every such pull request fails commitlint, is blocked from merging, and the
// dependency never lands - which is how a security update stops arriving.
//
// So the limit still applies, and it is measured over everything that could
// have been wrapped. A line is exempt only when one of its whitespace-
// separated tokens is itself longer than the limit: the line is long because
// of something unbreakable, not because nobody wrapped it. A 120-character
// line of ordinary words still fails, which is the case the rule is for.

const LIMIT = 100;

const offendingLines = (text) =>
  (text ?? '')
    .split('\n')
    .filter((line) => line.length > LIMIT)
    .filter((line) => !line.split(/\s+/).some((token) => token.length > LIMIT));

const wrappableLineLength = (section) => (parsed) => {
  const offenders = offendingLines(parsed[section]);
  const detail = offenders
    .map((line) => `  ${line.length} chars: ${line.slice(0, 60)}...`)
    .join('\n');
  return [
    offenders.length === 0,
    `${section} lines must not be longer than ${LIMIT} characters, unless one ` +
      `token on the line is longer than that on its own:\n${detail}`,
  ];
};

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
