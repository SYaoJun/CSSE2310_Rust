/* CSSE2310 2025 Semester Two Assignment One
 * Written by Eric Staykov
 * This version is personalised for s4903470 Haoyu ZHU
 */

// Header files to be included are below ///////////////////////////////////////

#include <ctype.h>
#include <getopt.h> // Only needed if USE_GETOPT symbol is defined
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <csse2310a1.h>

// To build this program with the getopt version of command line processing,
// uncomment the following line (or add -DUSE_GETOPT to the gcc command line)
// #define USE_GETOPT

// Constant definitions are below //////////////////////////////////////////////

// Constant characters
const char escapeKey = (char)27;
const char endTransmission = (char)4;
const char backspace = (char)127;
const char add = '+';
const char subtract = '-';
const char multiply = '*';
const char divide = '/';
const char newlineChar = '\n';
const char nullTerminator = '\0';
const char colon = ':';
const char changeInputBase = 'i';
const char changeOutputBase = 'o';
const char showHistory = 'h';
const char zero = '0';
const char comma = ',';

// Simple constant strings
const char* const emptyString = "";
const char* const newlineStr = "\n";
const char* const doubleDash = "--";

// Input and output bases default values and limits
const int decimalBase = 10;
const int defaultOutputBases[] = {2, 10, 16};
const int defaultOutputBaseCount = 3;
const int minBase = 2;
const int maxBase = 36;

// Max number of characters of a number
// Assume the system is 64-bit and can represent this
const int maxDigits = 64;

// Initial size of buffer when reading from input file
const int bufferLength = 32;

// Each history entry has expression, result and base elements
const int historyIncrement = 3;

// Print messages to the user
const char* const printExpression = "Expression (base %d): %s\n";
const char* const printExpressionStrBase = "Expression (base %s): %s\n";
const char* const printResult = "Result (base %d): %s\n";
const char* const printResultStrbase = "Result (base %s): %s\n";
const char* const printInput = "Input (base %d): %s\n";
const char* const printOutput = "Base %d: %s\n";
const char* const expressionError = "Can't evaluate the expression \"%s\"\n";

// Startup messages
const char* const welcomeMessage = "Welcome to uqbasejump!\n"
                                   "s4903470 wrote this program.\n";
const char* const welcomeInputBase = "Input base set to: ";
const char* const welcomeOutputBase = "Output bases: ";
const char* const welcomeLastLine
        = "Please enter your numbers and expressions.\n";

// Command line option arguments
const char* const inputBaseArg = "inbase";
const char* const outputBaseArg = "obases";
const char* const inputFileArg = "inputfile";

// Exit messages that result in program termination
const char* const okExitMessage = "Thanks for using uqbasejump.\n";
const char* const usageErrorMessage = "Usage: ./uqbasejump [--obases 2..36] "
                                      "[--inbase 2..36] [--inputfile string]\n";
const char* const fileErrorMessage
        = "uqbasejump: unable to read from file \"%s\"\n";
const char* const exitUnknownStatusMessage
        = "uqbasejump: unknown exit status\n";

// Enum definitions are below //////////////////////////////////////////////////

// This enum contains the program exit status codes
typedef enum {
    EXIT_OK_STATUS = 0,
    EXIT_USAGE_STATUS = 7,
    EXIT_INPUT_FILE_STATUS = 16,
} ExitStatus;

#ifdef USE_GETOPT
// This enum contains our argument types - used for the getopt() version of
// command line argument parsing
typedef enum { INPUT_ARG = 1, OUTPUT_ARG = 2, FILE_ARG = 3 } ArgType;
#endif

// Struct definitions are below ////////////////////////////////////////////////

// This struct contains the parameters extracted from the command line
typedef struct {
    int inputBase;
    int* outputBases;
    int outputBaseCount;
    char* inputFileName;
    FILE* inputFile;
} Arguments;

// This struct contains the input and expression strings, the history array
// and their lengths
typedef struct {
    char* input; // Buffer for current number input
    int inputLength; // Length of current input
    char* expr; // Buffer for arithmetic expression
    int exprLength;
    char** history;
    int historyLength;
} InputExpr;

// Function prototypes are below ///////////////////////////////////////////////
Arguments* init_arguments_struct(void);
InputExpr* init_input_expr_struct(void);
void free_input_expr_struct(InputExpr* inputExpr);
void reset_expression(InputExpr* inputExpr);
void reset_input(InputExpr* inputExpr);
Arguments* parse_command_line(int argc, char** argv);
void check_for_empty_string(char* toCheck, Arguments* args);
void cleanup_and_exit(Arguments* args, ExitStatus exitStatus);
int increment_and_check_arg_count(
        int count, int argc, char** argv, Arguments* args);
bool check_set_input_base(char* inputBaseStr, Arguments* args);
bool check_set_output_base(char* outputBaseStr, Arguments* args);
int check_base(char* baseStr);
bool check_duplicate_base(int value, const int* bases, int baseCount);
void open_input_file(Arguments* args);
void print_welcome_message(Arguments* args);
void get_check_input_file(Arguments* args);
char* read_line(FILE* stream);
void get_check_input_stdin(Arguments* args);
void process_expression(InputExpr* inputExpr, Arguments* args);
void handle_operator(InputExpr* inputExpr, Arguments* args, char operator);
void check_input_empty(InputExpr* inputExpr);
bool handle_command(InputExpr* inputExpr, Arguments* args);
void handle_alphanumeric_input(
        InputExpr* inputExpr, Arguments* args, char input);
void update_display(InputExpr* inputExpr, Arguments* args);
void print_in_bases(unsigned long long value, Arguments* args);

// Function definitions are below //////////////////////////////////////////////

/* main()
 * ------
 * Main function of the program. Parse the command line, check the specified
 * file, print the startup message, process numbers and expressions, and
 * cleanup and exit.
 *
 * argc: number of command line arguments (including program name)
 * argv: array of command line argument strings (including program name)
 *
 * Returns: an integer representing the exit status of the program
 * Global variables modified: none
 * Errors: if any errors occur, print the relevant error message and exit
 * with the relevant status; otherwise, print the exit OK message and exit
 * with status OK
 */
int main(int argc, char** argv)
{
    Arguments* args = parse_command_line(argc, argv);
    open_input_file(args);
    print_welcome_message(args);
    if (args->inputFile) {
        get_check_input_file(args);
    } else {
        get_check_input_stdin(args);
    }
    cleanup_and_exit(args, EXIT_OK_STATUS);
}

/* init_arguments_struct()
 * ------------------------
 * Initialise an Arguments struct with default values for input base, output
 * bases, and input file settings.
 *
 * Returns: pointer to the malloc'd Arguments struct with default values
 * Global variables modified: none
 * Errors: none
 */
Arguments* init_arguments_struct(void)
{
    Arguments* args = malloc(sizeof(Arguments));
    args->inputBase = decimalBase;
    args->outputBaseCount = defaultOutputBaseCount;
    args->outputBases = malloc(sizeof(int) * defaultOutputBaseCount);
    for (int i = 0; i < defaultOutputBaseCount; i++) {
        args->outputBases[i] = defaultOutputBases[i];
    }
    args->inputFileName = NULL;
    args->inputFile = NULL;
    return args;
}

/* init_input_expr_struct()
 * ------------------------
 * Initialise an InputExpr struct with default values for input buffer,
 * expression, and history tracking.
 *
 * Returns: pointer to the malloc'd InputExpr struct
 * Global variables modified: none
 * Errors: none
 */

InputExpr* init_input_expr_struct(void)
{
    InputExpr* inputExpr = malloc(sizeof(InputExpr));
    inputExpr->input = malloc(sizeof(char) * (maxDigits + 1));
    inputExpr->inputLength = 0;
    inputExpr->expr = NULL;
    inputExpr->exprLength = 0;
    reset_input(inputExpr);
    reset_expression(inputExpr);
    inputExpr->history = (char**)malloc(sizeof(char*));
    inputExpr->historyLength = 0;
    return inputExpr;
}

/* free_input_expr_struct()
 * ------------------------
 * Frees all memory allocated for an InputExpr struct including input,
 * expression, and history buffers.
 *
 * inputExpr: pointer to the InputExpr struct to be freed; assumed not NULL
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void free_input_expr_struct(InputExpr* inputExpr)
{
    if (inputExpr == NULL) {
        return;
    }
    free(inputExpr->input);
    free(inputExpr->expr);
    for (int i = 0; i < inputExpr->historyLength; i++) {
        free((inputExpr->history)[i]);
    }
    free((void*)(inputExpr->history));
    free(inputExpr);
}

/* reset_expression()
 * ------------------
 * Resets the expression buffer in the InputExpr struct to an empty string.
 *
 * inputExpr: pointer to the InputExpr struct whose expression is to be reset;
 * assumed not NULL
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void reset_expression(InputExpr* inputExpr)
{
    if (inputExpr == NULL) {
        return;
    }
    free(inputExpr->expr);
    inputExpr->expr = malloc(sizeof(char));
    (inputExpr->expr)[0] = nullTerminator;
    inputExpr->exprLength = 0;
}

/* reset_input()
 * -------------
 * Resets the input buffer in the InputExpr struct to an empty string.
 *
 * inputExpr: pointer to the InputExpr struct whose input is to be reset;
 * assumed not NULL
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void reset_input(InputExpr* inputExpr)
{
    if (inputExpr == NULL) {
        return;
    }
    (inputExpr->input)[0] = nullTerminator;
    inputExpr->inputLength = 0;
}

#ifdef USE_GETOPT
// getopt() version of command line argument processing. Compile with
// -DUSE_GETOPT argument to gcc to include this code. There is a non-getopt()
// version of parse_command_line() below.

/* parse_command_line()
 * --------------------
 * Parses command line arguments to extract input base, output bases, and input
 * file name.
 *
 * argc: number of command line arguments (including program name)
 * argv: array of command line argument strings
 *
 * Returns: pointer to an Arguments struct populated with parsed values
 * Global variables modified: none
 * Errors: print a usage error message and exit if any command line arguments
 * are incorrect or there are too many command line arguments or an option
 * is set more than once
 * REF: Code here is based on the example in the getopt(3) man page.
 */
Arguments* parse_command_line(int argc, char** argv)
{
    struct option longOptions[]
            = {{inputBaseArg, required_argument, NULL, INPUT_ARG},
                    {outputBaseArg, required_argument, NULL, OUTPUT_ARG},
                    {inputFileArg, required_argument, NULL, FILE_ARG},
                    {0, 0, 0, 0}}; // Last element should be all zeros
    int optionIndex = 0;
    Arguments* args = init_arguments_struct();
    bool inputBaseSet = false, outputBaseSet = false;
    while (true) {
        // Get the next option argument. (":" prevents error message printing)
        // ("+" means we stop processing when we hit a non-option argument)
        int opt = getopt_long(argc, argv, "+:", longOptions, &optionIndex);
        if (opt == -1) { // Ran out of option arguments
            break;
        }
        if ((opt == INPUT_ARG) && optarg) {
            // Got the input base argument
            if (inputBaseSet) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            if (!check_set_input_base(optarg, args)) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            inputBaseSet = true;
        } else if ((opt == OUTPUT_ARG) && optarg) {
            // Got the output base argument
            if (outputBaseSet) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            if (!check_set_output_base(optarg, args)) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            outputBaseSet = true;
        } else if ((opt == FILE_ARG) && optarg) {
            // Got the input file argument
            if (args->inputFileName) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            args->inputFileName = optarg;
        } else { // Unknown argument or no following option
            cleanup_and_exit(args, EXIT_USAGE_STATUS);
        }
    }
    if (optind < argc) { // Leftover arguments - error
        cleanup_and_exit(args, EXIT_USAGE_STATUS);
    }
    return args;
}

#else
// Non getopt() version of command line argument processing

/* parse_command_line()
 * --------------------
 * Parses command line arguments to extract input base, output bases, and input
 * file name.
 *
 * argc: number of command line arguments (including program name)
 * argv: array of command line argument strings
 *
 * Returns: pointer to an Arguments struct populated with parsed values
 * Global variables modified: none
 * Errors: print a usage error message and exit if any command line arguments
 * are incorrect or there are too many command line arguments or an option
 * is set more than once
 */
Arguments* parse_command_line(int argc, char** argv)
{
    argv++; // get rid of the program name
    argc--; // decrement the number of arguments
    Arguments* args = init_arguments_struct();
    bool inputBaseSet = false, outputBaseSet = false;
    for (int i = 0; i < argc; i++) { // Iterate over command line arguments
        check_for_empty_string(argv[i], args);
        if (strncmp(argv[i], doubleDash, strlen(doubleDash))) {
            // Argument doesn't start with --
            cleanup_and_exit(args, EXIT_USAGE_STATUS);
        }
        if (!strcmp(argv[i] + strlen(doubleDash), inputBaseArg)) {
            // Got the input base argument
            if (inputBaseSet) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            i = increment_and_check_arg_count(i, argc, argv, args);
            if (!check_set_input_base(argv[i], args)) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            inputBaseSet = true;
        } else if (!strcmp(argv[i] + strlen(doubleDash), outputBaseArg)) {
            // Got the output base argument
            if (outputBaseSet) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            i = increment_and_check_arg_count(i, argc, argv, args);
            if (!check_set_output_base(argv[i], args)) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            outputBaseSet = true;
        } else if (!strcmp(argv[i] + strlen(doubleDash), inputFileArg)) {
            // Got the input file argument
            if (args->inputFileName) {
                cleanup_and_exit(args, EXIT_USAGE_STATUS);
            }
            i = increment_and_check_arg_count(i, argc, argv, args);
            args->inputFileName = argv[i];
        } else {
            cleanup_and_exit(args, EXIT_USAGE_STATUS);
        }
    }
    return args;
}
#endif

/* check_for_empty_string()
 * -------------------------
 * Checks whether a given string is empty and exits the program with a usage
 * error if it is.
 *
 * toCheck: the string to check; assumed not NULL
 * args: pointer to an Arguments struct used for cleanup and error reporting
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: exits with usage error status if the string is empty
 */
void check_for_empty_string(char* toCheck, Arguments* args)
{
    if (!strcmp(toCheck, emptyString)) {
        cleanup_and_exit(args, EXIT_USAGE_STATUS);
    }
}

/* cleanup_and_exit()
 * ------------------
 * Cleans up allocated memory and exits the program with the specified exit
 * status.
 *
 * args: pointer to an Arguments struct containing allocated resources
 * exitStatus: the exit status to use when terminating the program
 *
 * Returns: nothing (program exits)
 * Global variables modified: none
 * Errors: prints appropriate error message based on exit status and exits
 */
void cleanup_and_exit(Arguments* args, ExitStatus exitStatus)
{
    if (exitStatus == EXIT_OK_STATUS) {
        fprintf(stdout, okExitMessage);
    } else if (exitStatus == EXIT_USAGE_STATUS) {
        fprintf(stderr, usageErrorMessage);
    } else if (exitStatus == EXIT_INPUT_FILE_STATUS) {
        fprintf(stderr, fileErrorMessage, args->inputFileName);
    } else {
        fprintf(stderr, exitUnknownStatusMessage);
    }
    if (args) {
        if (args->outputBases) {
            free((void*)(args->outputBases));
        }
        if (args->inputFile) {
            fclose(args->inputFile);
        }
        free(args);
    }
    exit(exitStatus);
}

/* increment_and_check_arg_count()
 * --------------------------------
 * Increments the argument index and checks if it exceeds the total number of
 * arguments. Also checks if the next argument is an empty string.
 *
 * count: current index in the argument list
 * argc: total number of command line arguments
 * argv: array of command line argument strings
 * args: pointer to an Arguments struct used for cleanup and error reporting
 *
 * Returns: updated argument index
 * Global variables modified: none
 * Errors: exits with usage error status if index exceeds argc or next argument
 * is empty
 */
int increment_and_check_arg_count(
        int count, int argc, char** argv, Arguments* args)
{
    count++;
    if (count >= argc) {
        cleanup_and_exit(args, EXIT_USAGE_STATUS);
    }
    check_for_empty_string(argv[count], args);
    return count;
}

/* check_set_input_base()
 * ----------------------
 * Validates and sets the input base from a string representation.
 *
 * inputBaseStr: string representing the input base
 * args: pointer to an Arguments struct to update the input base
 *
 * Returns: true if input base is valid and set; false otherwise
 * Global variables modified: none
 * Errors: none
 */
bool check_set_input_base(char* inputBaseStr, Arguments* args)
{
    int value = check_base(inputBaseStr);
    if (!value) {
        return false;
    }
    args->inputBase = value;
    return true;
}

/* check_set_output_base()
 * -----------------------
 * Parses and sets multiple output bases from a comma-separated string.
 *
 * outputBaseStr: string containing comma-separated output bases
 * args: pointer to an Arguments struct to update the output bases
 *
 * Returns: true if all output bases are valid and set; false otherwise,
 * e.g. if any token is invalid, duplicated, or empty
 * Global variables modified: none
 * Errors: none
 */
bool check_set_output_base(char* outputBaseStr, Arguments* args)
{
    // Save the old base array in case we need to restore it
    int* oldBases = args->outputBases;
    int baseCount = 0;
    args->outputBases = malloc(sizeof(int)); // Start with space for one base
    // Pointer to the beginning of the current token
    char* start = outputBaseStr;
    char* end = NULL;
    // Loop through each comma-separated token
    while ((end = strchr(start, comma))) {
        if (end == start) {
            // Empty token (e.g. ",," or leading comma)
            args->outputBases = oldBases; // Restore old base array
            return false;
        }
        // Copy the token into a temporary buffer
        char* token = strndup(start, end - start);
        // Validate the base value
        int value = check_base(token);
        if (!value
                || !check_duplicate_base(value, args->outputBases, baseCount)) {
            free(token);
            args->outputBases = oldBases; // Restore old base array
            return false;
        }
        // Store the valid base and resize the array
        args->outputBases[baseCount++] = value;
        args->outputBases
                = realloc(args->outputBases, sizeof(int) * (baseCount + 1));
        start = end + 1; // Move to the next token
        free(token);
    }
    // Handle the final token (after the last comma)
    if (*start == nullTerminator) {
        // Trailing comma indicates an empty token
        args->outputBases = oldBases;
        return false;
    }
    // Validate and store the final token
    int value = check_base(start);
    if (!value || !check_duplicate_base(value, args->outputBases, baseCount)) {
        args->outputBases = oldBases;
        return false;
    }
    args->outputBases[baseCount++] = value;
    args->outputBaseCount = baseCount;
    free((void*)oldBases); // Free the old base array
    return true;
}

/* check_base()
 * ------------
 * Converts a string to an integer and checks if it falls within the valid base
 * range.
 *
 * baseStr: string representing a base value
 *
 * Returns: integer value of base if valid; 0 if invalid
 * Global variables modified: none
 * Errors: none
 */
int check_base(char* baseStr)
{
    char* endPtr = NULL;
    long value = strtol(baseStr, &endPtr, decimalBase);
    if (*endPtr || (value > maxBase) || (value < minBase)) {
        return 0;
    }
    int length = strlen(baseStr);
    for (int i = 0; i < length; i++) {
        if (!isdigit(baseStr[i])) {
            return 0;
        }
    }
    return (int)value;
}

/* check_duplicate_base()
 * ----------------------
 * Checks whether a given base value already exists in the list of output bases.
 *
 * value: base value to check
 * bases: array of existing base values
 * baseCount: number of existing base values
 *
 * Returns: true if value is not a duplicate; false if it is
 * Global variables modified: none
 * Errors: none
 */
bool check_duplicate_base(int value, const int* bases, int baseCount)
{
    for (int i = 0; i < baseCount; i++) {
        if (bases[i] == value) {
            return false;
        }
    }
    return true;
}

/* open_input_file()
 * -----------------
 * Attempts to open the input file specified in the Arguments struct and stores
 * the file pointer.
 *
 * args: pointer to an Arguments struct containing the input file name
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: exits with input file error status if the file cannot be opened
 */
void open_input_file(Arguments* args)
{
    if (!(args->inputFileName)) {
        return;
    }
    FILE* file = fopen(args->inputFileName, "r");
    if (!file) {
        cleanup_and_exit(args, EXIT_INPUT_FILE_STATUS);
    }
    args->inputFile = file;
}

/* print_welcome_message()
 * -----------------------
 * Prints a welcome message including input and output base settings. Clears the
 * screen if no input file is provided.
 *
 * args: pointer to an Arguments struct containing base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: returns early if args is NULL
 */
void print_welcome_message(Arguments* args)
{
    if (!args) {
        return;
    }
    if (!(args->inputFile)) { // Don't clear the screen if an input file has
                              // been provided
        clear_screen();
    }
    printf(welcomeMessage);
    printf("%s%d\n", welcomeInputBase, args->inputBase);
    printf(welcomeOutputBase);
    for (int i = 0; i < args->outputBaseCount; i++) {
        printf("%d", args->outputBases[i]);
        if (i < (args->outputBaseCount - 1)) {
            printf(", ");
        } else {
            printf(newlineStr);
        }
    }
    if (!(args->inputFile)) {
        printf(welcomeLastLine);
    }
}

/* get_check_input_file()
 * ----------------------
 * Reads and processes expressions line-by-line from the input file. Converts
 * input to base 10, evaluates the expression, and prints results in all output
 * bases.
 *
 * args: pointer to an Arguments struct containing input file and base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: prints error message for invalid expressions and continues processing
 */
void get_check_input_file(Arguments* args)
{
    char* line = NULL;
    while ((line = read_line(args->inputFile))) {
        char* exprBaseTen
                = convert_expression(line, args->inputBase, decimalBase);
        unsigned long long result = 0;
        int error = evaluate_expression(exprBaseTen, &result);
        if (error) {
            fprintf(stderr, expressionError, line);
        } else {
            char* exprInputBase = convert_expression(
                    line, args->inputBase, args->inputBase);
            char* resultConverted
                    = convert_int_to_str_any_base(result, args->inputBase);
            printf(printExpression, args->inputBase, exprInputBase);
            printf(printResult, args->inputBase, resultConverted);
            print_in_bases(result, args); // Show result in all output bases
            free(exprInputBase);
            free(resultConverted);
        }
        free(exprBaseTen);
        free(line);
    }
}

/* read_line()
 * -----------
 * Reads the next line from a file stream into a dynamically allocated buffer.
 *
 * stream: the file stream to read from
 *
 * Returns: pointer to the read line or NULL if EOF or error occurs
 * Global variables modified: none
 * Errors: none
 * REF: Ed Lessons Week 3.2 – file handling (Basic file handling)
 */
char* read_line(FILE* stream)
{
    int bufferSize = bufferLength;
    char* buffer = malloc(sizeof(char) * bufferSize);
    int numRead = 0;
    int next;
    // Return if the stream is EOF
    if (feof(stream)) {
        free(buffer);
        return NULL;
    }
    while (true) {
        // Get the next char in the stream
        next = fgetc(stream);
        // Return NULL if the stream is EOF and we didn't read anything
        if (next == EOF && numRead == 0) {
            free(buffer);
            return NULL;
        }
        // Check if we hit the current buffer limit
        if (numRead == (bufferSize - 1)) {
            // Increase the buffer size and allocate more memory for the buffer
            bufferSize *= 2;
            buffer = realloc(buffer, sizeof(char) * bufferSize);
        }
        // Return what we have if we hit a newline or EOF
        if (next == newlineChar || next == EOF) {
            buffer[numRead] = nullTerminator;
            break;
        }
        // Add the char to the buffer
        buffer[numRead++] = next;
    }
    return buffer;
}

/* get_check_input_stdin()
 * -----------------------
 * Reads and processes input character-by-character from stdin. Handles
 * commands, operators, and alphanumeric input, updating the display
 * accordingly.
 *
 * args: pointer to an Arguments struct containing base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: exits loop on EOF or end-of-transmission character
 */
void get_check_input_stdin(Arguments* args)
{
    disable_line_buffering();
    InputExpr* inputExpr = init_input_expr_struct();
    while (true) {
        bool updateDisplay = true;
        int input = fgetc(stdin); // Read one char at a time
        if (input == EOF) {
            // get EOF when line buffering is enabled
            break;
        }
        char c = (char)input;
        if (c == endTransmission) {
            // get endTransmission when line buffering is disabled
            break;
        }
        if (c == escapeKey) { // Clear input and expression
            reset_expression(inputExpr);
            reset_input(inputExpr);
            clear_screen();
        } else if (c == newlineChar) { // Evaluate expression
            process_expression(inputExpr, args);
            updateDisplay = false;
        } else if (c == backspace) { // Delete last character
            if (inputExpr->inputLength > 0) {
                (inputExpr->input)[--(inputExpr->inputLength)] = nullTerminator;
            }
        } else if (c == add || c == subtract || c == multiply || c == divide) {
            // Operator entered: append current input and operator to expression
            handle_operator(inputExpr, args, c);
        } else if (c == colon) {
            updateDisplay = handle_command(inputExpr, args);
        } else if (isalnum(c)) {
            handle_alphanumeric_input(inputExpr, args, c);
        }
        if (updateDisplay) {
            update_display(inputExpr, args);
        }
    }
    free_input_expr_struct(inputExpr);
}

/* process_expression()
 * --------------------
 * Converts current input to base-10, appends to expression, evaluates it, and
 * prints results in all output bases. Adds the expression and result to
 * history.
 *
 * inputExpr: pointer to InputExpr struct containing current input and
 * expression args: pointer to Arguments struct containing base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: prints error message if expression evaluation fails
 */
void process_expression(InputExpr* inputExpr, Arguments* args)
{
    check_input_empty(inputExpr);
    char* baseTenInput
            = convert_any_base_to_base_ten(inputExpr->input, args->inputBase);
    inputExpr->exprLength += strlen(baseTenInput);
    inputExpr->expr = realloc(
            inputExpr->expr, sizeof(char) * (inputExpr->exprLength + 1));
    strcat(inputExpr->expr, baseTenInput); // Append current input to expression
    free(baseTenInput);
    unsigned long long result = 0;
    int error = evaluate_expression(inputExpr->expr, &result);
    if (error) {
        fprintf(stderr, expressionError, inputExpr->expr);
    } else {
        clear_screen();
        char* exprConverted = convert_expression(
                inputExpr->expr, decimalBase, args->inputBase);
        printf(printExpression, args->inputBase, exprConverted);
        char* resultConverted
                = convert_int_to_str_any_base(result, args->inputBase);
        printf(printResult, args->inputBase, resultConverted);
        print_in_bases(result, args); // Show result in all output bases
        // add expression and result and base to history
        inputExpr->history = (char**)realloc((void*)(inputExpr->history),
                sizeof(char*) * (inputExpr->historyLength + historyIncrement));
        (inputExpr->history)[inputExpr->historyLength] = exprConverted;
        (inputExpr->history)[inputExpr->historyLength + 1] = resultConverted;
        (inputExpr->history)[inputExpr->historyLength + 2]
                = convert_int_to_str_any_base(args->inputBase, decimalBase);
        inputExpr->historyLength += historyIncrement;
    }
    // Reset for next input
    reset_expression(inputExpr);
    reset_input(inputExpr);
}

/* handle_operator()
 * -----------------
 * Converts current input to base 10, appends it and the operator to the
 * expression, and resets the input buffer.
 *
 * inputExpr: pointer to InputExpr struct containing current input and
 * expression args: pointer to Arguments struct containing base settings
 * operator: arithmetic operator character to append
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void handle_operator(InputExpr* inputExpr, Arguments* args, char operator)
{
    check_input_empty(inputExpr);
    char* baseTenInput
            = convert_any_base_to_base_ten(inputExpr->input, args->inputBase);
    inputExpr->exprLength += strlen(baseTenInput);
    // +2 because going to add the operator before NULL terminator
    inputExpr->expr = realloc(
            inputExpr->expr, sizeof(char) * (inputExpr->exprLength + 2));
    strcat(inputExpr->expr, baseTenInput);
    free(baseTenInput);
    inputExpr->expr[inputExpr->exprLength++] = operator;
    inputExpr->expr[inputExpr->exprLength] = nullTerminator;
    reset_input(inputExpr);
}

/* check_input_empty()
 * -------------------
 * Checks if the current input buffer is the empty string. If it is, adds '0'.
 *
 * inputExpr: pointer to InputExpr struct containing current input and
 * expression args: pointer to Arguments struct containing base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void check_input_empty(InputExpr* inputExpr)
{
    if (!strlen(inputExpr->input)) {
        (inputExpr->input)[inputExpr->inputLength++] = zero;
        (inputExpr->input)[inputExpr->inputLength] = nullTerminator;
    }
}

/* handle_command()
 * ----------------
 * Handles command input for changing input/output bases or displaying history.
 *
 * inputExpr: pointer to InputExpr struct containing current input and history
 * args: pointer to Arguments struct containing base settings
 *
 * Returns: true if display should be updated; false otherwise
 * Global variables modified: none
 * Errors: none
 */
bool handle_command(InputExpr* inputExpr, Arguments* args)
{
    // Command mode: change input or output bases
    int input = fgetc(stdin); // Read another char
    if (input == EOF) {
        // get EOF when line buffering is enabled
        return true;
    }
    char c = (char)input;
    if (c == endTransmission) {
        // get endTransmission when line buffering is disabled
        return true;
    }
    bool success = false;
    char* line = read_line(stdin);
    if (!line) {
        return true;
    }
    if (c == changeInputBase) {
        success = check_set_input_base(line, args);
    } else if (c == changeOutputBase) {
        success = check_set_output_base(line, args);
    } else if (c == showHistory && (!strlen(line))) {
        clear_screen();
        for (int i = 0; i < inputExpr->historyLength; i += historyIncrement) {
            printf(printExpressionStrBase, (inputExpr->history)[i + 2],
                    (inputExpr->history)[i]);
            printf(printResultStrbase, (inputExpr->history)[i + 2],
                    (inputExpr->history)[i + 1]);
        }
        free(line);
        return false; // don't update the display
    }
    if (success) {
        // Reset input and expression
        reset_expression(inputExpr);
        reset_input(inputExpr);
    }
    free(line);
    return true;
}

/* handle_alphanumeric_input()
 * ---------------------------
 * Validates and appends alphanumeric input to the current input buffer if
 * within base range and digit limit.
 *
 * inputExpr: pointer to InputExpr struct containing current input
 * args: pointer to Arguments struct containing input base
 * input: character to validate and append
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void handle_alphanumeric_input(
        InputExpr* inputExpr, Arguments* args, char input)
{
    char str[2];
    str[0] = input;
    str[1] = nullTerminator;
    long value = strtol(str, NULL, maxBase);
    // Valid character for number input
    if ((inputExpr->inputLength < maxDigits)
            && (value < (long)(args->inputBase))) {
        (inputExpr->input)[inputExpr->inputLength++] = input;
        (inputExpr->input)[inputExpr->inputLength] = nullTerminator;
    }
}

/* update_display()
 * ----------------
 * Clears the screen and displays the current expression, input, and converted
 * values in all output bases.
 *
 * inputExpr: pointer to InputExpr struct containing current input and
 * expression args: pointer to Arguments struct containing base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void update_display(InputExpr* inputExpr, Arguments* args)
{
    clear_screen();
    char* exprConverted
            = convert_expression(inputExpr->expr, decimalBase, args->inputBase);
    printf(printExpression, args->inputBase, exprConverted);
    free(exprConverted);
    printf(printInput, args->inputBase, inputExpr->input);
    unsigned long long value = 0;
    if (strlen(inputExpr->input)) {
        value = convert_str_to_int_any_base(inputExpr->input, args->inputBase);
    }
    print_in_bases(value, args);
}

/* print_in_bases()
 * ----------------
 * Converts and prints a numeric value in all selected output bases.
 *
 * value: the numeric value to convert and print
 * args: pointer to Arguments struct containing output base settings
 *
 * Returns: nothing
 * Global variables modified: none
 * Errors: none
 */
void print_in_bases(unsigned long long value, Arguments* args)
{
    for (int i = 0; i < args->outputBaseCount; i++) {
        char* result = convert_int_to_str_any_base(value, args->outputBases[i]);
        printf(printOutput, args->outputBases[i], result);
        free(result);
    }
}
