// Modifier keys
export type Modifier = 'ctrl' | 'alt' | 'shift' | 'meta';

// Alphabetic keys (a-z)
export type AlphaKey =
	| 'a'
	| 'b'
	| 'c'
	| 'd'
	| 'e'
	| 'f'
	| 'g'
	| 'h'
	| 'i'
	| 'j'
	| 'k'
	| 'l'
	| 'm'
	| 'n'
	| 'o'
	| 'p'
	| 'q'
	| 'r'
	| 's'
	| 't'
	| 'u'
	| 'v'
	| 'w'
	| 'x'
	| 'y'
	| 'z';

// Numeric keys (0-9)
export type NumericKey = '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';

// Function keys (F1-F24)
export type FunctionKey =
	`f${1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24}`;

// Navigation keys
export type NavigationKey = 'home' | 'end' | `arrow${'up' | 'down' | 'left' | 'right'}`;

// Lock keys
export type LockKey = `${'num' | 'caps' | 'scroll' | ''}lock`;

// Special character keys
export type SpecialCharKey =
	| 'semicolon'
	| 'equals'
	| 'comma'
	| 'dash'
	| 'period'
	| 'slash'
	| 'backquote'
	| 'openbracket'
	| 'backslash'
	| 'closebracket'
	| 'quote'
	| 'plus'
	| 'minus';

// Action keys
export type ActionKey = 'escape' | 'tab' | 'space' | 'enter' | 'backspace' | 'delete' | 'pause';

// All base keys combined
export type BaseKey =
	| AlphaKey
	| NumericKey
	| FunctionKey
	| NavigationKey
	| LockKey
	| SpecialCharKey
	| ActionKey;

// Key combination export types
export type SingleKey = Modifier | BaseKey;
export type ModifierCombo = `${Modifier}+${BaseKey}` | `${Modifier}+${Modifier}+${BaseKey}`;
export type KeyCombination = SingleKey | ModifierCombo;
