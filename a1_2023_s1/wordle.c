#include<stdio.h>
#include<stdlib.h>
#include<string.h>
#include<stdbool.h>
#include<ctype.h>
#include<csse2310a1.h>

/*defines*/
#define OK 0
#define USAGE 1
#define FILE_ERR 2
#define GAME_OVER 3

#define DFLT_FILE "/usr/share/dict/words"
#define DFLT_LEN 5
#define DFLT_MAX 6

#define LEN "-len"
#define MAX "-max"
#define EMPT " "

/*struct defines*/
typedef struct {
    long wordlength;
    long maxguess;
    char* dictionary;
    char* ans;
    char* prog;
    char** stored;
    int dictnum;
} Wordle;

/*function defines*/
/*game functions*/
void start_game(Wordle* wordle);
void set_game(Wordle* wordle);
void store_dict(Wordle* wordle);
int verify_input(char* input, long wordlength, int guesses);
int verify_w_dict(char* input, char* fname, int guesses, long wordlength);
int verify_w_dict_stored(char* input, char** dict, int guesses,
        int dictnum, long wordlength);
int guess_check(char* guess, Wordle* wordle);
void reset_prog(Wordle* wordle);
void send_msg_welcome(void);
void send_msg_game_prog(long length, long guessleft);
void send_msg_invalid(void);
void send_msg_length(long length);
void send_msg_unknown_word(void);
void send_msg_gameover(bool correct, char* secret);
/*setting of default values for game*/
void default_info(Wordle* wordle);
/*arg check functions*/
void arg_check(int argc, char** argv);
void arg_check_one(char** argv);
void arg_check_two(char** argv);
void arg_check_three(char** argv);
void arg_check_four(char** argv);
void arg_check_five(char** argv);
void check_five_len(int a, int b, int c, int d, int e, char** argv);
void check_five_max(int a, int b, int c, int d, int e, char** argv);
void check_five_dict(int a, int b, int c, int d, int e, char** argv);
/*handle arg inputs functions*/
void arg_process(int argc, char** argv, Wordle* wordle);
void handle_one(char** argv, Wordle* wordle);
void handle_two(char** argv, Wordle* wordle);
void handle_three(char** argv, Wordle* wordle);
void handle_four(char** argv, Wordle* wordle);
void handle_five(char** argv, Wordle* wordle);
/*exit status function*/
int exit_status(int status, char* fname);
/*helper functions*/
void dict_file_check(char* fname);
int option_check(char* option);
char* lowercase_word(char* word);
void check_len(char* argv);
void check_max(char* argv);
long get_num(char* str);
char* get_line(FILE* fname);

int main(int argc, char** argv) {

    /*initialise struct for wordle*/
    Wordle* wordle = malloc(sizeof(Wordle));

    /*set up default values for wordle*/
    default_info(wordle);

    /*check input args*/
    arg_check(argc, argv);

    /*handle args */
    arg_process(argc, argv, wordle);

    /*set game info*/
    set_game(wordle);

    /*start game*/
    start_game(wordle);

    return OK;
}

/*
 * @brief: main game function
 * @param: Wordle* wordle
 * @retval: none
 * */
void start_game(Wordle* wordle) {
    send_msg_welcome();

    char line[60];
    char* lowercase;
    int verify, guess, guesses;
    guesses = wordle->maxguess;
    send_msg_game_prog(wordle->wordlength, wordle->maxguess);

    while (1) {
        if (guesses == 0 || guess < 0) {
            free(wordle->stored);
            send_msg_gameover(0, wordle->ans);
        }
        fgets(line, 60, stdin);
        if (feof(stdin)) {
            free(wordle->stored);
            send_msg_gameover(0, wordle->ans);
        }

        lowercase = lowercase_word(line);
        verify = verify_input(line, wordle->wordlength, guesses);
        if (verify == 1) {
            continue;
        }
        if (verify == 0) { //verify by file/stored char array
            /*verify = verify_w_dict(lowercase, wordle->dictionary,
                    guesses, wordle->wordlength);*/
            verify = verify_w_dict_stored(lowercase, wordle->stored,
                    guesses, wordle->dictnum, wordle->wordlength);
            if (verify == 1) {
                continue;
            }
        }
        //guess should be valid by here.
        guess = guess_check(line, wordle);
        if (guess == 1) {
            send_msg_gameover(true, wordle->ans);
            break;
        }
        if (guess == 0) {
            guesses--;
            if (guesses == 0) {
                continue;
            } else {
                send_msg_game_prog(wordle->wordlength, guesses);
                continue;
            }
        }
    }
}

/*
 * @brief: sets game info
 * @param: Wordle* wordle
 * @retval: none;
 * */
void set_game(Wordle* wordle) {

    /*set answer word with provided library*/
    wordle->ans = get_random_word(wordle->wordlength);

    /*set progress word*/
    wordle->prog = (char*)malloc(sizeof(char) * wordle->wordlength + 1);
    reset_prog(wordle);

    /*store dictionary*/
    store_dict(wordle);
}

/*
 * @brief: stores dictionary to memory
 * @param: Wordle* wordle
 * @retval: none
 * */
void store_dict(Wordle* wordle) {

    int size = 10;
    int i = 0;
    int ptr = 2;
    char buff[60];

    FILE* file = fopen(wordle->dictionary, "r");
    /*assumed from previous checks file is does not have problems*/

    wordle->stored = malloc(sizeof(char*) * size);

    while(fgets(buff, 60, file) != NULL ) {

        int len;
        buff[(len = strcspn( buff, "\n"))] = 0;

        if (i == ptr) {
            void* tmp = realloc(wordle->stored, (2 * ptr) * sizeof
                    * wordle->stored);
            wordle->stored = tmp;
            ptr *= 2;
        }

        if (!(wordle->stored[i] = malloc (len + 1))) {
            break;
        }

        memcpy (wordle->stored[i], buff, len + 1);

        i++;
    }

    wordle->dictnum = i;

    fclose(file);

}

/*
 * @brief: function to verify user input
 * @param: char* input, long wordlength, int guesses
 * @retval: int
 * */
int verify_input(char* input, long wordlength, int guesses) {

    /*check if length is correct*/
    int strlength;

    strlength = strlen(input) - 1;

    if (strlength != wordlength || strlength == 0) {
        send_msg_length(wordlength);
        send_msg_game_prog(wordlength, guesses);
        return 1;
    }

    /*check if contains numbers*/
    int i, check;

    for (i = 0; i < wordlength; i++) {
        check = isalpha(input[i]);
        if (check == 0) {
            send_msg_invalid();
            send_msg_game_prog(wordlength, guesses);
            return 1;
        }
    }

    return 0;

}

/*
 * @brief: function to verify input with dictionary
 * @param: char* input, char* fname, int guesses, long wordlength.
 * @retval: int
 * */
int verify_w_dict(char* input, char* fname, int guesses, long wordlength) {

    int flag;
    char line[50];

    flag = 1;
    /*open file*/
    FILE* file = fopen(fname, "r");

    while ((fgets(line, 50, file) != NULL)) {
        if (strstr(line, input) != NULL) {
            flag = 0;
            break;
        }
    }

    if (flag == 1) {
        send_msg_unknown_word();
        send_msg_game_prog(wordlength, guesses);
    }

    fclose(file);

    return flag;

}

/*
 * @brief: function to verify input with stored dictionary
 * @param: char* input, char** dict, int guesses, int dictnum, long wordlength
 * @retval: int
 * */
int verify_w_dict_stored(char* input, char** dict, int guesses,
        int dictnum, long wordlength) {

    int flag;
    int i = 0;
    int result;
    char* word;


    flag = 1;

    for (i = 0; i < dictnum; i++) {
        word = lowercase_word(dict[i]);
        result = strncmp(word, input, wordlength);
        if (result == 0) {
            /*word exists*/
            flag = 0;
            break;
        }
    }

    if (flag == 1) {
        send_msg_unknown_word();
        send_msg_game_prog(wordlength, guesses);
    }

    return flag;
}

/*
 * @brief: function to  check with answer
 * @param: char* guess, Wordle* wordle
 * @retval: int
 * */
int guess_check(char* guess, Wordle* wordle) {
    int flag = 0;
    int x = 0;
    int y = 0;
    char* answer;
    char* anscpy;

    answer = lowercase_word(wordle->ans);
    anscpy = (char*)malloc(sizeof(char) * wordle->wordlength);
    reset_prog(wordle);
    strcpy(anscpy, answer);
    /*if letter is correct position*/
    for (x = 0; x < wordle->wordlength; x++) {
        if (guess[x] == answer[x]) {
            wordle->prog[x] = toupper(guess[x]);
            anscpy[x] = '-';
        }
    }
    /*check if any other letters are in answer but in incorrect position*/
    for (x = 0; x < wordle->wordlength; x++) {
        if (wordle->prog[x] == '-') {
            for (y = 0; y < wordle->wordlength; y++) {
                if (guess[x] == anscpy[y]) {
                    wordle->prog[x] = guess[x];
                    break;
                }
            }
        }
    }
    /*checking if guess is correct.*/
    for (x = 0; x < wordle->wordlength; x++) {
        if (isupper(wordle->prog[x])) {
            flag = 1;
            continue;
        } else {
            flag = 0;
            break;
        }
    }
    free(anscpy);
    if (flag == 1) {
        return 1;
    } else {
        fprintf(stdout, "%s\n", wordle->prog);
        return 0;
    }
}

/*
 * @brief: resets progress string -> the ---- string
 * @param: Wordle* wordle
 * @retval: none;
 * */
void reset_prog(Wordle* wordle) {

    memset(wordle->prog, '-', wordle->wordlength);
    wordle->prog[wordle->wordlength] = '\0';

}

/*
 * @brief: sends welcome message
 * @param: none
 * @retval: none
 * */
void send_msg_welcome(void) {

    fprintf(stdout, "Welcome to Wordle!\n");

}

/*
 * @brief: send msg of game prog - word length and attempts remaining
 * @param: long length, long guessleft
 * @retval: none
 * */
void send_msg_game_prog(long length, long guessleft) {

    if (guessleft >= 2) {
        fprintf(stdout, "Enter a %ld letter word (%ld attempts remaining):\n",
                length, guessleft);
    } else {
        if (guessleft < 2) {
            fprintf(stdout, "Enter a %ld letter word (last attempt):\n",
                    length);
        }
    }

}

/*
 * @brief: send msg of invalid attempt
 * @param: none
 * @retval: none
 * */
void send_msg_invalid(void) {

    fprintf(stdout, "Words must contain only letters - try again.\n");

}

/*
 * @brief: send msg for incorrect word length in guess
 * @param: long length
 * @retval: none
 * */
void send_msg_length(long length) {

    fprintf(stdout, "Words must be %ld letters long - try again.\n", length);

}

/*
 * @brief: send msg of unknown word
 * @param: none
 * @retval: none
 * */
void send_msg_unknown_word(void){

    fprintf(stdout, "Word not found in the dictionary - try again.\n");

}

/*
 * @brief: send msg of game over;
 * @param: bool correct, char* secret
 * @retval: none
 * */
void send_msg_gameover(bool correct, char* secret) {

    if (correct) {
        printf("Correct!\n");
        exit_status(OK, EMPT);
    } else {
        fprintf(stderr, "Bad luck - the word is \"%s\".\n", secret);
        exit(GAME_OVER);
    }
}

/*
 * @brief: function to populate game info with default values
 * @param: Wordle* wordle
 * @retval: Wordle*
 * */
void default_info(Wordle* wordle) {
    wordle->wordlength = DFLT_LEN;
    wordle->maxguess = DFLT_MAX;
    wordle->dictionary = DFLT_FILE;
}

/*
 * @brief: function to check input arguments
 * @param: int argc, char** argv
 * @retval: none
 * */
void arg_check(int argc, char** argv) {

    /*checks number of inputs into program*/
    if (argc > 6) {
        exit_status(USAGE, EMPT);
    }

    if (argc == 2) {
        arg_check_one(argv);
    }

    if (argc == 3) {
        arg_check_two(argv);
    }

    if (argc == 4) {
        arg_check_three(argv);
    }

    if (argc == 5) {
        arg_check_four(argv);
    }

    if (argc == 6) {
        arg_check_five(argv);
    }

}

/*
 * @brief: checks arg when 1 is inputted
 * @param: char** argv
 * @retval: none
 * */
void arg_check_one(char** argv) {

    int a;

    a = option_check(argv[1]);

    /*checks if argument is -len or -max if yes -> USAGE*/
    if (a == 1 || a == 2) {
        exit_status(USAGE, EMPT);
    } else {
        /*if its not -len or -max then must be dict*/
        if (argv[1][0] == '-') {
            exit_status(USAGE, EMPT);
        }
        dict_file_check(argv[1]);
    }

}

/*
 * @brief: checks arg when 2 is inputted
 * @param: char** argv
 * @retval: none
 * */
void arg_check_two(char** argv) {
    /*
    ./wordle -len x || ./wordle -max x --> valid
    ./wordle (dict) -len || ./wordle (dict) -max --> USAGE
    */
    /*opt, num*/
    int a, b;

    a = option_check(argv[1]);
    b = option_check(argv[2]);

    /*check if option is LEN*/
    if (a == 1) {
        /*check if argv[2] is a number or not*/
        check_len(argv[2]);
    }

    /*check if option is MAX*/
    if (a == 2) {
        check_max(argv[2]);
    }

    /*does not max either len or max so must be dict file-->USAGE error*/
    if (a == 0) {
        exit_status(USAGE, EMPT);
    }

    /*./wordle dict -option --> USAGE error*/
    if (b == 1 || b == 2) {
        exit_status(USAGE, EMPT);
    }

}

/*
 * @brief: checks arg when 3 is inputted
 * @param: char** argv
 * @retval: none
 * */
void arg_check_three(char** argv) {

    /*
    ./wordle -len x -max || ./wordle -max x -len --> USAGE
    ./wordle (dict) -len x -->valid
    ./wordle (dict) -max x --> valid
    ./wordle -len x (dict) --> valid
    ./wordle -max x (dict) --> valid
    */

    int a, b, c;

    a = option_check(argv[1]);
    b = option_check(argv[2]);
    c = option_check(argv[3]);
    /*arg[1] = -len --> argv[2] & argv[3] should not be -options*/
    if (a == 1) {
        /*checks if the argv[2]/[3] is an -option input --> USAGE error*/
        if (b != 0 || c != 0) {
            exit_status(USAGE, EMPT);
        }
        check_len(argv[2]);
        dict_file_check(argv[3]);
    }

    /*same as above but with -max option as first input*/
    if (a == 2) {
        if (b != 0 || c != 0) {
            exit_status(USAGE, EMPT);
        }
        check_max(argv[2]);
        dict_file_check(argv[3]);
    }

    /*argv[1] must be dict file*/
    if (a == 0) {
        /*making sure argv[2] is -option if [1] is dict*/
        if (b == 1) {
            check_len(argv[3]);
        } else if (b == 2) {
            check_max(argv[3]);
        } else {
            exit_status(USAGE, EMPT);
        }

        dict_file_check(argv[1]);
    }
}

/*
 * @brief: checks arg when 4 is inputted
 * @param: char** argv
 * @retval: none
 * */
void arg_check_four(char** argv) {

    /*
    ./wordle -len x -max y || ./wordle -max x -len y --> valid
    ./wordle -len x -len y || ./wordle -max x -max y --> USAGE
    */

    int a, b, c, d;

    a = option_check(argv[1]);
    b = option_check(argv[2]);
    c = option_check(argv[3]);
    d = option_check(argv[4]);

    /*./wordle -len x -max y*/
    if (a == 1) {
        check_len(argv[2]);
        /*./wordle -len x -len y --> USAGE */
        if (a == c) {
            exit_status(USAGE, EMPT);
        }
        if (c == 2) {
            check_max(argv[4]);
        }
    }

    /*./wordle -max x -len y*/
    if (a == 2) {
        check_max(argv[2]);
        if (a == c) {
            exit_status(USAGE, EMPT);
        }
        if (c == 1) {
            check_len(argv[4]);
        }
    }

    if (a == 0 || c == 0) {
        exit_status(USAGE, EMPT);
    }

    if (b == 1 || b == 2 || d == 1 || d == 2) {
        exit_status(USAGE, EMPT);
    }

}

/*
 * @brief: checks arg when 5 is inputted
 * @param: char** argv
 * @retval: none
 * */
void arg_check_five(char** argv) {
    /*
    ./wordle -len x -max y (dict) || ./wordle -len x (dict) -max y --> valid
    ./wordle (dict) -len x -max y --> valid
    ./wordle -max x -len y (dict) || ./wordle -max x (dict) -len y --> valid
    ./wordle (dict) -max x -len y
    */

    int a, b, c, d, e;

    a = option_check(argv[1]);
    b = option_check(argv[2]);
    c = option_check(argv[3]);
    d = option_check(argv[4]);
    e = option_check(argv[5]);

    if (a == 0 && b == 0 && c == 0 && d == 0 && e == 0) {
        exit_status(USAGE, EMPT);
    }

    check_five_len(a, b, c, d, e, argv);
    check_five_max(a, b, c, d, e, argv);
    check_five_dict(a, b, c, d, e, argv);

}

/*
 * @brief: helper function to check when there are 5 inputs to program
 *          with -len being first option input
 * @param: int a, int b, int c, int d, int e, char** argv
 * @retval: none
 * */
void check_five_len(int a, int b, int c, int d, int e, char** argv) {

    if (a == 1) {
        check_len(argv[2]);
        if (a == c || a == e) {
            exit_status(USAGE, EMPT);
        }
        if (c == 2) {
            check_max(argv[4]);
            if (argv[5][0] == '-') {
                exit_status(USAGE, EMPT);
            }
            dict_file_check(argv[5]);
        }
        if (c == 0) {
            if (argv[3][0] == '-') {
                exit_status(USAGE, EMPT);
            }
            if (d == 2) {
                check_max(argv[5]);
            } else {
                exit_status(USAGE, EMPT);
            }
            dict_file_check(argv[3]);
        }
    }

}

/*
 * @brief: helper function to check when there are 5 inputs to program
 *          with -max being first option input
 * @param: int a, int b, int c, int d, int e, char** argv
 * @retval: none
 * */
void check_five_max(int a, int b, int c, int d, int e, char** argv) {

    if (a == 2) {
        check_max(argv[2]);
        if (a == c || a == e) {
            exit_status(USAGE, EMPT);
        }
        if (c == 1) {
            check_len(argv[4]);
            if (argv[5][0] == '-') {
                exit_status(USAGE, EMPT);
            }
            dict_file_check(argv[5]);
        }
        if (c == 0) {
            if (argv[3][0] == '-') {
                exit_status(USAGE, EMPT);
            }
            if (d == 1) {
                check_len(argv[5]);
            } else {
                exit_status(USAGE, EMPT);
            }
            dict_file_check(argv[3]);
        }
    }

}

/*
 * @brief: helper function to check when there are 5 inputs to program
 *          with (dict) being first input
 * @param: int a, int b, int c, int d, int e, char** argv
 * @retval: none
 * */
void check_five_dict(int a, int b, int c, int d, int e, char** argv) {

    if (a == 0) {
        if (argv[1][0] == '-') {
            exit_status(USAGE, EMPT);
        }
        if (b == 1) {
            if (b == d) {
                exit_status(USAGE, EMPT);
            }
            check_len(argv[3]);
            if (d == 2) {
                check_max(argv[5]);
            }
        } else if (b == 2) {
            if (b == d) {
                exit_status(USAGE, EMPT);
            }
            check_max(argv[3]);
            if (d == 1) {
                check_len(argv[5]);
            }
        } else {
            exit_status(USAGE, EMPT);
        }
        dict_file_check(argv[1]);
    }

}

/*
 * @brief: function to handle input arguments for gameplay
 * @param: int argc, char** argc, Wordle* wordle
 * @retval: none
 * */
void arg_process(int argc, char** argv, Wordle* wordle) {

    if (argc == 2) {
        handle_one(argv, wordle);
    }

    if (argc == 3) {
        handle_two(argv, wordle);
    }

    if (argc == 4) {
        handle_three(argv, wordle);
    }

    if (argc == 5) {
        handle_four(argv, wordle);
    }

    if (argc == 6) {
        handle_five(argv, wordle);
    }

}

/*
 * @brief: function to handle 1 argument
 * @param: char** argv, Wordle* wordle
 * @retval: none
 * */
void handle_one(char** argv, Wordle* wordle) {

    wordle->dictionary = argv[1];

}

/*
 * @brief: function to handle 2 arguments
 * @param: char** argv, Wordle* wordle
 * @retval: none
 * */
void handle_two(char** argv, Wordle* wordle) {

    int a;
    long num;

    a = option_check(argv[1]);
    num = get_num(argv[2]);
    if (a == 1) {
        wordle->wordlength = num;
    }

    if (a == 2) {
        wordle->maxguess = num;
    }

}

/*
 * @brief: function to handle 3 arguments
 * @param: char** argv, Wordle* wordle
 * @retval: none
 * */
void handle_three(char** argv, Wordle* wordle) {

    int a, b;
    long num;

    a = option_check(argv[1]);
    b = option_check(argv[2]);

    /*if first option is (dict) file*/
    if (a == 0) {
        wordle->dictionary = argv[1];
        num = get_num(argv[3]);
        if (b == 1) {
            wordle->wordlength = num;
        }
        if (b == 2) {
            wordle->maxguess = num;
        }
    }

    /*if first option is either -len or -max*/
    num = get_num(argv[2]);
    if (a == 1) {
        wordle->wordlength = num;
    }
    if (a == 2) {
        wordle->maxguess = num;
    }
    wordle->dictionary = argv[3];

}

/*
 * @brief: function to handle 4 arguments
 * @param: char** argv, Wordle* wordle
 * @retval: none
 * */
void handle_four(char** argv, Wordle* wordle) {

    int a;
    long x, y;

    a = option_check(argv[1]);

    x = get_num(argv[2]);
    y = get_num(argv[4]);

    if (a == 1) {
        wordle->wordlength = x;
        wordle->maxguess = y;
    }
    if (a == 2) {
        wordle->maxguess = x;
        wordle->wordlength = y;
    }

}

/*
 * @brief:function to handle 5 arguments
 * @param: char** argv, Wordle* wordle
 * @retval: none
 * */
void handle_five(char** argv, Wordle* wordle) {
    int a, b, c;
    long x, y;

    a = option_check(argv[1]);
    b = option_check(argv[2]);
    c = option_check(argv[3]);

    /*if first arg is dictionary*/
    if (a == 0) {
        x = get_num(argv[3]);
        y = get_num(argv[5]);
        wordle->dictionary = argv[1];
        if (b == 1) {
            wordle->wordlength = x;
            wordle->maxguess = y;
        }
        if (b == 2) {
            wordle->maxguess = x;
            wordle->wordlength = y;
        }
    }

    x = get_num(argv[2]);
    if (a == 1) {
        wordle->wordlength = x;
        if (c == 2) {
            y = get_num(argv[4]);
            wordle->maxguess = y;
            wordle->dictionary = argv[5];
        }
        if (c == 0) {
            wordle->dictionary = argv[3];
            y = get_num(argv[5]);
            wordle->maxguess = y;
        }
    }
    if (a == 2) {
        wordle->maxguess = x;
        if (c == 1) {
            y = get_num(argv[4]);
            wordle->wordlength = y;
            wordle->dictionary = argv[5];
        }
        if (c == 0) {
            wordle->dictionary = argv[3];
            y = get_num(argv[5]);
            wordle->wordlength = y;
        }
    }
}

/*
 * @brief: function to handle exit status
 * @param: int status, char* fname
 * @retval: int
 * */
int exit_status(int status, char* fname) {

    switch (status) {
        case USAGE:
            fprintf(stderr, "Usage: wordle [-len word-length] "
                        "[-max max-guesses] [dictionary]\n");
            exit(USAGE);
        case FILE_ERR:
            fprintf(stderr, "wordle: dictionary file \"%s\" "
                        "cannot be opened\n", fname);
            exit(FILE_ERR);
        default:
            exit(OK);

    }

}

/*
 * @brief: checks dict file
 * @param: char* fname
 * @retval: none
 * */
void dict_file_check(char* fname) {

    FILE* file = fopen(fname, "r");

    if (!file) {

        exit_status(FILE_ERR, fname);

    }

    fclose(file);

}

/*
 * @brief: checks option arg
 * @param: char* option
 * @retval: int
 * */
int option_check(char* option) {

    if (strcmp(option, LEN) == 0) {
        return 1;
    } else if (strcmp(option, MAX) == 0) {
        return 2;
    } else {
        return 0;
    }

}

/*
 * @brief: makes entire string into lowercase
 * @param: char* word
 * @retval: char*
 * */
char* lowercase_word(char* word) {

    int length;

    length = strlen(word);

    for (int i = 0; i < length; i++) {

        word[i] = (char)tolower((int)word[i]);

    }

    return word;

}

/*
 * @brief: check len value
 * @param: char* argv
 * @retval: none
 * */
void check_len(char* argv) {

    char* ptr;
    long num;

    num = strtol(argv, &ptr, 10);
    /*making sure its not a string such as 3ab20*/
    if (strlen(ptr) != 0) {
        exit_status(USAGE, EMPT);
    }

    /*checking if number for len is between 3 and 9*/
    if (num < 3 || num > 9) {
        exit_status(USAGE, EMPT);
    }

}

/*
 * @brief: check max value
 * @param: char* argv
 * @retval: none
 * */
void check_max(char* argv) {

    char* ptr;
    long num;

    num = strtol(argv, &ptr, 10);
    if (strlen(ptr) != 0) {
        exit_status(USAGE, EMPT);
    }

    if (num < 3 || num > 9) {
        exit_status(USAGE, EMPT);
    }
}

/*
 * @brief: function to get number for -max or -len from string
 * @param: char* argv
 * @retval: long
 * */
long get_num(char* str) {

    char* ptr;
    long num;

    num = strtol(str, &ptr, 10);

    return num;

}

/*
 * @brief: function to get line from file
 * @param: FILE* fname
 * @retval: char*
 * */
char* get_line(FILE* fname) {

    char* line;
    int size = 10;
    int c;
    int i = 0;

    line = (char*)malloc(sizeof(char) * size);

    while (((c = fgetc(fname)) != EOF) && (c != '\n')) {
        line[i] = c;
        if (i == size) {
            size += 10;
            line = (char*)realloc(line, sizeof(char) * size);
        }
        i++;
    }

    return line;
}