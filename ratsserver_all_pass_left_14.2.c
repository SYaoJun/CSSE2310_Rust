#include "csse2310a4.h"
#include <ctype.h>
#include <netdb.h>
#include <pthread.h>
#include <sched.h>
#include <semaphore.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>

/* Command line arguments */
const char *const localhost = "localhost";
const int minArgsSize = 3;
const int maxArgsSize = 4;
const int maxIntSize = 5;
const int maxConnectNum = 10000;

/* Game constants */
const int maxRound = 13;
const int maxTricks = 13;

/* Buffer and card sizes */
#define CARD_SIZE 256
#define DATA_SIZE 1024
#define TEMP_SIZE 64
/* Player and card distribution */
#define MAX_PLAYER_NUM 4
#define MAX_CARD_NUM 32
const int maxDeckSize = 104;
const int singleCardGroup = 8;

/* Card encoding constants */
const int cardAce = 14;
const int cardKing = 13;
const int cardQueen = 12;
const int cardJack = 11;
const int cardTen = 10;
const int cardMinNum = 2;
const int cardMaxNum = 9;

/* Card distribution indices */
const int playerZeroMod = 0;
const int playerOneMod = 2;
const int playerTwoMod = 4;
const int playerThreeMod = 6;

/* Team indices */
const int teamOneFirstPlayer = 0;
const int teamOneSecondPlayer = 2;
const int teamTwoFirstPlayer = 1;
const int teamTwoSecondPlayer = 3;
const int playerThreeIndex = 3;
/* String lengths */
const int nameLength = 10;
const int handCardStep = 2;
const int minRecvLen = 2;

/* Initialization values */
const int initialTurn = 0;
const int initialLeading = 0;
const int initialPlayerIdx = 0;

typedef struct {
  int maxconns;
  char *port;
  char *message;
} Arguments;

typedef enum {
  IDLE = 0,
  WATTING = 1,
  READY = 2,
  PLAYING = 3,
  COMPLETED = 4
} State;

typedef struct ClientInfo{
  struct ClientInfo *next;
  int fd;
  char name[DATA_SIZE];
  char gameName[DATA_SIZE];
  State state;
  int idx;
  char hand[MAX_CARD_NUM];
} ClientInfo;

typedef struct GameInfo{
  struct GameInfo *next;
  State state;
  char gameName[DATA_SIZE];
  ClientInfo *players[MAX_PLAYER_NUM];
  int playerCount;
  int currentTurn;
  int leadingPlayer;
  char suit;
  int playCards[MAX_PLAYER_NUM];
  int count;
  int teamOneTricks;
  int teamTwoTricks;
  int countReady;
  pthread_cond_t cond;
  pthread_mutex_t lock;
} GameInfo;

typedef struct ServerContext{
  pthread_mutex_t clientLock;
  pthread_mutex_t gameLock;
  pthread_mutex_t contextLock;
  ClientInfo *clientList;
  GameInfo *pendingGameList;
  int gameCount;
  int clientCount;
  int connected;
  int completed;
  int terminated;
  int tricks;
  int running;
  sem_t *conn;
} ServerContext;

typedef struct ThreadArgs{
  int fd;
  ServerContext *ctx;
  ClientInfo *clientInfo;
  char *message;
} ThreadArgs;

typedef enum {
  EXIT_SYSTEM_STATUS = 20,
  EXIT_PORT_STATUS = 17,
  EXIT_USAGE_STATUS = 8,
} ExitStatus;

void show_usage(void) {
  fprintf(stderr, "Usage: ./ratsserver maxconns message [portnum]\n");
  exit(EXIT_USAGE_STATUS);
}

void print_port_error(const char *port) {
  fprintf(stderr, "ratsserver: cannot listen on given port \"%s\"\n", port);
  exit(EXIT_PORT_STATUS);
}

/* is_number()
 * -----------
 * Checks if a given string represents a valid number.
 *
 * str: the string to check
 *
 * Returns: true if the string is a valid number, false otherwise
 * Global variables modified: none
 * Errors: none
 */
bool is_number(const char *str) {
  int n = strlen(str);
  if (n == 0 || n > maxIntSize) {
    return false;
  }
  for (int i = 0; i < n; i++) {
    if (i == 0 && str[i] == '+') {
      continue;
    }
    if (!isdigit(str[i])) {
      return false;
    }
  }
  return true;
}

/* parse_command_line_arguments()
 * -----------------------------
 * Parses command line arguments to extract max connections, message, and port number.
 *
 * argc: number of command line arguments (including program name)
 * argv: array of command line argument strings
 * args: pointer to an Arguments struct to populate with parsed values
 *
 * Returns: none
 * Global variables modified: none
 * Errors: prints a usage error message and exits if arguments are invalid
 */
void parse_command_line_arguments(int argc, char *argv[], Arguments *args) {
  if (argc < minArgsSize || argc > maxArgsSize) {
    show_usage();
  }
  for (int i = 1; i < argc; i++) {
    if (strlen(argv[i]) == 0) {
      show_usage();
    }
  }
  int maxConns = atoi(argv[1]);
  if (maxConns < 0 || maxConns > maxConnectNum) {
    show_usage();
  }
  args->maxconns = (maxConns == 0) ? maxConnectNum : maxConns;
  args->message = argv[2];
}

/* send_to_client()
 * ----------------
 * Sends a message to a client via a file descriptor.
 *
 * fd: the file descriptor of the client socket
 * message: the message to send
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
/* send_to_client()
 * ----------------
 * Sends a message to a client via a file descriptor.
 *
 * fd: the file descriptor of the client socket
 * message: the message to send
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void send_to_client(int fd, char *message) {
  send(fd, message, strlen(message), MSG_NOSIGNAL);
}

/* send_team_info()
 * ----------------
 * Sends team information to a client.
 *
 * fd: the file descriptor of the client socket
 * game: pointer to the GameInfo struct containing team details
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
/* send_team_info()
 * ----------------
 * Sends team information to a client.
 *
 * fd: the file descriptor of the client socket
 * game: pointer to the GameInfo struct containing team details
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void send_team_info(int fd, GameInfo *game) {
  char message[CARD_SIZE];
  char *playerOneName = game->players[teamOneFirstPlayer]->name;
  char *playerThreeName = game->players[teamOneSecondPlayer]->name;
  sprintf(message, "MTeam 1: %s, %s\n", playerOneName, playerThreeName);
  send_to_client(fd, message);

  char *playerTwoName = game->players[teamTwoFirstPlayer]->name;
  char *playerFourName = game->players[teamTwoSecondPlayer]->name;
  memset(message, 0, CARD_SIZE * sizeof(char));
  sprintf(message, "MTeam 2: %s, %s\n", playerTwoName, playerFourName);
  send_to_client(fd, message);
}

/* send_hand_and_start()
 * ---------------------
 * Sends hand information to a client and starts the game.
 *
 * fd: the file descriptor of the client socket
 * hands: the hand information to send
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
/* send_hand_and_start()
 * ---------------------
 * Sends hand information to a client and starts the game.
 *
 * fd: the file descriptor of the client socket
 * hands: the hand information to send
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void send_hand_and_start(int fd, char *hands) {
  char message[CARD_SIZE];
  sprintf(message, "H%s\n", hands);
  send_to_client(fd, message);

  memset(message, 0, CARD_SIZE * sizeof(char));
  strcpy(message, "MStarting the game\n");
  send_to_client(fd, message);
}

/* deal_cards()
 * ------------
 * Sends team information and hand details to a client to start the game.
 *
 * game: pointer to the GameInfo struct containing game details
 * hands: the hand information to send
 * fd: the file descriptor of the client socket
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void deal_cards(GameInfo *game, char hands[MAX_CARD_NUM], int fd) {
  send_team_info(fd, game);
  send_hand_and_start(fd, hands);
}

/* distribute_cards_to_player()
 * -----------------------------
 * Distributes cards from the deck to a player's hand based on the starting modulus.
 *
 * deck: the deck of cards to distribute from
 * hands: the array of player hands to populate
 * playerIdx: the index of the player to distribute cards to
 * startMod: the starting modulus for card distribution
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void distribute_cards_to_player(const char *deck,
                                char hands[MAX_PLAYER_NUM][MAX_CARD_NUM],
                                int playerIdx, int startMod) {
  int j = 0;
  for (int i = 0; i < maxDeckSize; i++) {
    if (i % singleCardGroup == startMod ||
        i % singleCardGroup == (startMod + 1)) {
      hands[playerIdx][j] = deck[i];
      j++;
    }
  }
}

/* play_games()
 * ------------
 * Initializes a game by distributing cards to players and starting the game.
 *
 * clientFd: the file descriptor of the client socket
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the current client
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void play_games(int clientFd, GameInfo *game,
                ClientInfo *client) {
  char *deck = get_random_deck();
  char hands[MAX_PLAYER_NUM][MAX_CARD_NUM];
  memset(hands, 0, MAX_PLAYER_NUM * MAX_CARD_NUM * sizeof(char));

  distribute_cards_to_player(deck, hands, 0, playerZeroMod);
  distribute_cards_to_player(deck, hands, 1, playerOneMod);
  distribute_cards_to_player(deck, hands, 2, playerTwoMod);
  distribute_cards_to_player(deck, hands, playerThreeIndex, playerThreeMod);

  strcpy(client->hand, hands[client->idx]);
  deal_cards(game, hands[client->idx], clientFd);
  free(deck);
}

/* check_port()
 * ------------
 * Checks if a given port is available and binds to it.
 *
 * port: the port number to check and bind to
 *
 * Returns: the socket file descriptor if successful
 * Global variables modified: none
 * Errors: prints an error message and exits if the port is invalid or unavailable
 */
int check_port(const char *port) {
  struct addrinfo hints;
  struct addrinfo *ai = NULL;
  memset(&hints, 0, sizeof(hints));
  hints.ai_family = AF_INET;
  hints.ai_socktype = SOCK_STREAM;
  hints.ai_flags = AI_PASSIVE;
  const char *realPort = (port == NULL) ? "0" : port;

  if (getaddrinfo(NULL, realPort, &hints, &ai) != 0) {
    print_port_error(realPort);
  }

  int serv = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
  if (serv < 0) {
    freeaddrinfo(ai);
    print_port_error(realPort);
  }

  if (bind(serv, ai->ai_addr, ai->ai_addrlen) != 0) {
    close(serv);
    freeaddrinfo(ai);
    print_port_error(realPort);
  }

  struct sockaddr_in addr;
  socklen_t len = sizeof(addr);
  if (getsockname(serv, (struct sockaddr *)&addr, &len) == 0) {
    fprintf(stderr, "%d\n", ntohs(addr.sin_port));
    fflush(stderr);
  }

  if (listen(serv, BUFSIZ) != 0) {
    close(serv);
    freeaddrinfo(ai);
    print_port_error(realPort);
  }

  freeaddrinfo(ai);
  return serv;
}

/* compare()
 * ---------
 * Compares two ClientInfo structs by their names for sorting purposes.
 *
 * a: pointer to the first ClientInfo struct
 * b: pointer to the second ClientInfo struct
 *
 * Returns: an integer less than, equal to, or greater than zero if the first
 *          argument is considered to be respectively less than, equal to, or
 *          greater than the second
 * Global variables modified: none
 * Errors: none
 */
int compare(const void *a, const void *b) {
  const ClientInfo *structA = *(const ClientInfo **)a;
  const ClientInfo *structB = *(const ClientInfo **)b;
  return strcmp(structA->name, structB->name);
}

/* create_new_game()
 * ------------------
 * Creates a new GameInfo struct and initializes its fields.
 *
 * gameName: the name of the new game
 *
 * Returns: a pointer to the newly created GameInfo struct
 * Global variables modified: none
 * Errors: none
 */
GameInfo *create_new_game(const char *gameName) {
  GameInfo *newGame = (GameInfo *)malloc(sizeof(GameInfo));
  newGame->state = IDLE;
  newGame->next = NULL;
  pthread_mutex_init(&newGame->lock, NULL);
  pthread_cond_init(&newGame->cond, NULL);
  newGame->playerCount = 0;
  newGame->count = 0;
  newGame->countReady = 0;
  newGame->teamOneTricks = 0;
  newGame->teamTwoTricks = 0;
  strcpy(newGame->gameName, gameName);
  return newGame;
}

/* find_or_create_game()
 * -----------------------
 * Finds an existing game with the given name or creates a new one if none exists.
 *
 * ctx: pointer to the ServerContext struct containing game list
 * gameName: the name of the game to find or create
 *
 * Returns: a pointer to the found or newly created GameInfo struct
 * Global variables modified: none
 * Errors: none
 */
GameInfo *find_or_create_game(ServerContext *ctx, const char *gameName) {
  GameInfo *currGame = ctx->pendingGameList;
  while (currGame != NULL) {
    if (strcmp(currGame->gameName, gameName) == 0 &&
        currGame->playerCount != MAX_PLAYER_NUM) {
      return currGame;
    }
    currGame = currGame->next;
  }

  GameInfo *newGame = create_new_game(gameName);
  newGame->next = (struct GameInfo*)ctx->pendingGameList;
  ctx->pendingGameList = newGame;
  ctx->gameCount++;
  return newGame;
}

/* setup_full_game()
 * -----------------
 * Sets up a game when all players have joined and are ready to play.
 *
 * game: pointer to the GameInfo struct to set up
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void setup_full_game(GameInfo *game) {
  game->state = READY;
  qsort((void*)game->players, MAX_PLAYER_NUM, sizeof(ClientInfo *),
        compare);
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    game->players[i]->idx = i;
  }
  game->leadingPlayer = initialLeading;
  game->currentTurn = initialTurn;
  game->count = 0;
  pthread_cond_broadcast(&game->cond);
}

/* match_players()
 * ----------------
 * Matches a client to a game based on their input and game name.
 *
 * ctx: pointer to the ServerContext struct containing game list
 * client: pointer to the ClientInfo struct for the current client
 * clientFd: the file descriptor of the client socket
 *
 * Returns: a pointer to the GameInfo struct the client was matched to
 * Global variables modified: none
 * Errors: closes the client socket and exits the thread if input is invalid
 */
GameInfo *match_players(ServerContext *ctx, ClientInfo *client, int clientFd) {
  char *input = read_line_fd(clientFd);
  if (input == NULL) {
    return NULL;
  }
  strcpy(client->name, input);
  free(input);

  input = read_line_fd(clientFd);
  if (input == NULL) {
    return NULL;
  }
  strcpy(client->gameName, input);
  free(input);

  pthread_mutex_lock(&ctx->gameLock);
  GameInfo *newGame = find_or_create_game(ctx, client->gameName);
  pthread_mutex_unlock(&ctx->gameLock);

  pthread_mutex_lock(&newGame->lock);
  newGame->players[newGame->playerCount] = client;
  newGame->playerCount++;

  if (newGame->playerCount == MAX_PLAYER_NUM) {
    setup_full_game(newGame);
  } else {
    pthread_cond_wait(&newGame->cond, &newGame->lock);
  }
  pthread_mutex_unlock(&newGame->lock);

  return newGame;
}

/* decode()
 * --------
 * Decodes a card number into its corresponding character representation.
 *
 * num: the card number to decode
 *
 * Returns: the character representation of the card number
 * Global variables modified: none
 * Errors: returns 0 if the number is invalid
 */
char decode(int num) {
  if (num == cardAce)
    return 'A';
  if (num == cardKing)
    return 'K';
  if (num == cardQueen)
    return 'Q';
  if (num == cardJack)
    return 'J';
  if (num == cardTen)
    return 'T';
  if (num >= cardMinNum && num <= cardMaxNum)
    return '0' + num;
  return 0;
}

/* encode()
 * --------
 * Encodes a card character into its corresponding numerical value.
 *
 * c: the card character to encode
 *
 * Returns: the numerical value of the card character
 * Global variables modified: none
 * Errors: returns 0 if the character is invalid
 */
int encode(char c) {
  if (c == 'A')
    return cardAce;
  if (c == 'K')
    return cardKing;
  if (c == 'Q')
    return cardQueen;
  if (c == 'J')
    return cardJack;
  if (c == 'T')
    return cardTen;
  if (c >= '2' && c <= '9')
    return c - '0';
  return 0;
}

/* check_input_validation()
 * -------------------------
 * Validates if a card and suit combination exists in a player's hand.
 *
 * hand: the player's hand of cards
 * suit: the suit to validate
 * card: the card to validate
 *
 * Returns: true if the card and suit combination is valid, false otherwise
 * Global variables modified: none
 * Errors: none
 */
bool check_input_validation(char hand[], char suit, char card) {
  int len = strlen(hand);
  for (int i = 0; i < len; i += handCardStep) {
    if (hand[i] == card && hand[i + 1] == suit) {
      return true;
    }
  }
  return false;
}

/* remove_card()
 * -------------
 * Removes a card from a player's hand.
 *
 * client: pointer to the ClientInfo struct for the player
 * cardNum: the numerical value of the card to remove
 * suit: the suit of the card to remove
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void remove_card(ClientInfo *client, int cardNum, char suit) {
  char cardChar = decode(cardNum);
  char *hand = client->hand;
  int len = strlen(hand);
  for (int i = 0; i < len; i += handCardStep) {
    if (hand[i] == cardChar && hand[i + 1] == suit) {
      for (int j = i; j < len - handCardStep; j++) {
        hand[j] = hand[j + handCardStep];
      }
      hand[len - handCardStep] = '\0';
      break;
    }
  }
}

/* broadcast_invalid_play()
 * -------------------------
 * Broadcasts a message to all players about an invalid play or early disconnect.
 *
 * game: pointer to the GameInfo struct containing game details
 * clientFd: the file descriptor of the client socket that disconnected
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void broadcast_invalid_play(GameInfo *game, int clientFd) {
  char tempBuffer[DATA_SIZE];
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    if (game->players[i]->fd != clientFd) {
      int playerNum = 0;
      for (int j = 0; j < MAX_PLAYER_NUM; j++) {
        if (game->players[j]->fd == clientFd) {
          playerNum = j + 1;
          break;
        }
      }
      memset(tempBuffer, 0, DATA_SIZE * sizeof(char));
      sprintf(tempBuffer, "Mplayer%d disconnected early\n", playerNum);
      send_to_client(game->players[i]->fd, tempBuffer);
      send_to_client(game->players[i]->fd, "O\n");
    }
  }
}

/* handle_early_disconnect()
 * --------------------------
 * Handles the early disconnection of a player from the game.
 *
 * game: pointer to the GameInfo struct containing game details
 * clientFd: the file descriptor of the client socket that disconnected
 * ctx: pointer to the ServerContext struct containing server state
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void handle_early_disconnect(GameInfo *game, int clientFd, ServerContext *ctx) {
  broadcast_invalid_play(game, clientFd);
  pthread_mutex_lock(&game->lock);
  game->state = COMPLETED;
  pthread_cond_broadcast(&game->cond);
  pthread_mutex_unlock(&game->lock);

  pthread_mutex_lock(&ctx->contextLock);
  ctx->terminated++;
  ctx->running--;
  pthread_mutex_unlock(&ctx->contextLock);
}

/* send_play_notification()
 * -------------------------
 * Sends a notification to all players about a card played by a client.
 *
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the player
 * recvMsg: the message containing the card played
 * clientFd: the file descriptor of the client socket
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void send_play_notification(GameInfo *game, ClientInfo *client, char *recvMsg,
                            int clientFd) {
  char sendBuffer[DATA_SIZE];
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    if (game->players[i]->fd != clientFd) {
      memset(sendBuffer, 0, DATA_SIZE * sizeof(char));
      char name[TEMP_SIZE];  
      char msg[CARD_SIZE]; 
      strncpy(name, client->name, sizeof(name)-1);
      strncpy(msg, recvMsg, sizeof(msg)-1);
      name[sizeof(name)-1] = '\0';
      msg[sizeof(msg)-1] = '\0';
      snprintf(sendBuffer, sizeof(sendBuffer), "M%s plays %s\n", name, msg);
      send_to_client(game->players[i]->fd, sendBuffer);
    } else {
      send_to_client(clientFd, "A\n");
      remove_card(client, encode(recvMsg[0]), recvMsg[1]);
    }
  }
}

/* update_play_state()
 * --------------------
 * Updates the game state after a player plays a card.
 *
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the player
 * recvMsg: the message containing the card played
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void update_play_state(GameInfo *game, ClientInfo *client, char *recvMsg) {
  if (game->leadingPlayer == client->idx) {
    game->suit = recvMsg[1];
  }

  if (game->suit != recvMsg[1]) {
    game->playCards[game->count] = 0;
  } else {
    game->playCards[game->count] = encode(recvMsg[0]);
  }

  game->currentTurn = (game->currentTurn + 1) % MAX_PLAYER_NUM;
  game->count++;
}

/* find_trick_winner()
 * -------------------
 * Determines the winner of a trick based on the cards played.
 *
 * game: pointer to the GameInfo struct containing game details
 *
 * Returns: the index of the player who won the trick
 * Global variables modified: none
 * Errors: none
 */
int find_trick_winner(GameInfo *game) {
  int maxValue = game->playCards[0];
  int winnerOffset = 0;
  for (int i = 1; i < MAX_PLAYER_NUM; i++) {
    if (game->playCards[i] > maxValue) {
      maxValue = game->playCards[i];
      winnerOffset = i;
    }
  }
  return (game->leadingPlayer + winnerOffset) % MAX_PLAYER_NUM;
}

/* update_tricks_and_notify()
 * --------------------------
 * Updates the trick count and notifies all players about the trick winner.
 *
 * game: pointer to the GameInfo struct containing game details
 * ctx: pointer to the ServerContext struct containing server state
 * winner: the index of the player who won the trick
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void update_tricks_and_notify(GameInfo *game, ServerContext *ctx, int winner) {
  memset(game->playCards, 0, MAX_PLAYER_NUM * sizeof(int));
  game->leadingPlayer = winner;
  game->currentTurn = winner;
  game->count = 0;

  if (winner == teamOneFirstPlayer || winner == teamOneSecondPlayer) {
    game->teamOneTricks++;
  } else {
    game->teamTwoTricks++;
  }

  char winMsg[CARD_SIZE];
  sprintf(winMsg, "MP%d won\n", winner + 1);
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    send_to_client(game->players[i]->fd, winMsg);
  }

  pthread_mutex_lock(&ctx->contextLock);
  ctx->tricks++;
  pthread_mutex_unlock(&ctx->contextLock);
}

/* announce_game_winner()
 * -----------------------
 * Announces the winner of the game to all players.
 *
 * game: pointer to the GameInfo struct containing game details
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void announce_game_winner(GameInfo *game) {
  char sendBuffer[DATA_SIZE];
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    memset(sendBuffer, 0, DATA_SIZE * sizeof(char));
    if (game->teamOneTricks > game->teamTwoTricks) {
      sprintf(sendBuffer, "MWinner is Team 1 (%d tricks won)\n",
              game->teamOneTricks);
    } else {
      sprintf(sendBuffer, "MWinner is Team 2 (%d tricks won)\n",
              game->teamTwoTricks);
    }
    send_to_client(game->players[i]->fd, sendBuffer);
  }
}

/* end_game_normally()
 * --------------------
 * Ends the game normally by announcing the winner and updating server state.
 *
 * game: pointer to the GameInfo struct containing game details
 * ctx: pointer to the ServerContext struct containing server state
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void end_game_normally(GameInfo *game, ServerContext *ctx) {
  announce_game_winner(game);
  for (int i = 0; i < MAX_PLAYER_NUM; i++) {
    send_to_client(game->players[i]->fd, "O\n");
  }
  game->state = COMPLETED;
  pthread_cond_broadcast(&game->cond);

  pthread_mutex_lock(&ctx->contextLock);
  ctx->running--;
  ctx->terminated++;
  ctx->completed++;
  pthread_mutex_unlock(&ctx->contextLock);
}

/* wait_for_turn()
 * ---------------
 * Waits for a player's turn to play in the game.
 *
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the player
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void wait_for_turn(GameInfo *game, ClientInfo *client) {
  while (game->currentTurn != client->idx && game->state != COMPLETED) {
    pthread_cond_wait(&game->cond, &game->lock);
  }
}

/* send_turn_prompt()
 * -------------------
 * Sends a prompt to a player indicating it is their turn to play.
 *
 * clientFd: the file descriptor of the client socket
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the player
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void send_turn_prompt(int clientFd, GameInfo *game, ClientInfo *client) {
  if (game->leadingPlayer == client->idx) {
    send_to_client(clientFd, "L\n");
  } else {
    char sendBuffer[DATA_SIZE];
    memset(sendBuffer, 0, DATA_SIZE * sizeof(char));
    sprintf(sendBuffer, "P%c\n", game->suit);
    send_to_client(clientFd, sendBuffer);
  }
}

/* main_game_loop()
 * -----------------
 * The main loop for handling game play, including turns and trick resolution.
 *
 * clientFd: the file descriptor of the client socket
 * game: pointer to the GameInfo struct containing game details
 * client: pointer to the ClientInfo struct for the player
 * ctx: pointer to the ServerContext struct containing server state
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void main_game_loop(int clientFd, GameInfo *game, ClientInfo *client,
                    ServerContext *ctx) {
  while (1) {
    pthread_mutex_lock(&game->lock);
    wait_for_turn(game, client);

    if (game->state == COMPLETED) {
      pthread_mutex_unlock(&game->lock);
      break;
    }

    send_turn_prompt(clientFd, game, client);

    char *recvMsg = read_line_fd(clientFd);
    if (recvMsg == NULL) {
      pthread_mutex_unlock(&game->lock);
      handle_early_disconnect(game, clientFd, ctx);
      break;
    }

    int len = strlen(recvMsg);
    if (len != minRecvLen ||
        !check_input_validation(client->hand, recvMsg[1], recvMsg[0])) {
      free(recvMsg);
      pthread_mutex_unlock(&game->lock);
      continue;
    }

    send_play_notification(game, client, recvMsg, clientFd);
    update_play_state(game, client, recvMsg);

    if (game->count == MAX_PLAYER_NUM) {
      int winner = find_trick_winner(game);
      update_tricks_and_notify(game, ctx, winner);

      if (game->teamOneTricks + game->teamTwoTricks == maxTricks) {
        end_game_normally(game, ctx);
        free(recvMsg);
        break;
      }
    }

    free(recvMsg);
    pthread_cond_broadcast(&game->cond);
    pthread_mutex_unlock(&game->lock);
  }
}

/* wait_all_ready()
 * ----------------
 * Waits for all players to be ready before starting the game.
 *
 * game: pointer to the GameInfo struct containing game details
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void wait_all_ready(GameInfo *game, ServerContext *ctx) {
  pthread_mutex_lock(&game->lock);
  game->countReady++;
  if (game->countReady < MAX_PLAYER_NUM) {
    while (game->countReady < MAX_PLAYER_NUM) {
      pthread_cond_wait(&game->cond, &game->lock);
    }
  } else {
      pthread_mutex_lock(&ctx->contextLock);
    ctx->running++;
    pthread_mutex_unlock(&ctx->contextLock);
    pthread_cond_broadcast(&game->cond);
  }
  pthread_mutex_unlock(&game->lock);
}

/* handle_new_connection()
 * ------------------------
 * Handles a new client connection, including game setup and play.
 *
 * args: pointer to the ThreadArgs struct containing thread arguments
 *
 * Returns: NULL
 * Global variables modified: none
 * Errors: none
 */
void *handle_new_connection(void *args) {
  ThreadArgs *threadArgs = (ThreadArgs *)args;
  ServerContext *ctx = threadArgs->ctx;
  ClientInfo *client = threadArgs->clientInfo;
  int clientFd = threadArgs->fd;

  char input[DATA_SIZE];
  memset(input, 0, DATA_SIZE * sizeof(char));
  sprintf(input, "M%s\n", threadArgs->message);
  send_to_client(clientFd, input);

  GameInfo *game = match_players(ctx, client, clientFd);
  if (game != NULL) {
    play_games(clientFd,  game, client);
    wait_all_ready(game, ctx);
    main_game_loop(clientFd, game, client, ctx);
  }

  pthread_mutex_lock(&ctx->contextLock);
  ctx->connected--;
  pthread_mutex_unlock(&ctx->contextLock);
  close(clientFd);
  free(threadArgs);

  sem_post(ctx->conn);
  return NULL;
}

/* init_server()
 * ------------
 * Initializes the server context with default values.
 *
 * Returns: a pointer to the newly initialized ServerContext struct
 * Global variables modified: none
 * Errors: none
 */
ServerContext *init_server() {
  ServerContext *ctx = (ServerContext *)malloc(sizeof(ServerContext));
  pthread_mutex_init(&ctx->contextLock, NULL);
  ctx->clientCount = 0;
  ctx->completed = 0;
  ctx->connected = 0;
  ctx->running = 0;
  ctx->terminated = 0;
  ctx->tricks = 0;
  ctx->clientList = NULL;
  ctx->pendingGameList = NULL;
  pthread_mutex_init(&ctx->clientLock, NULL);
  pthread_mutex_init(&ctx->gameLock, NULL);
  return ctx;
}

/* print_statistics()
 * ------------------
 * Prints server statistics to stderr.
 *
 * server: pointer to the ServerContext struct containing server state
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void print_statistics(ServerContext *server) {
  pthread_mutex_lock(&server->contextLock);
  fprintf(stderr, "Players connected: %d\n", server->connected);
  fprintf(stderr, "Total connected players: %d\n", server->clientCount);
  fprintf(stderr, "Running games: %d\n", server->running);
  fprintf(stderr, "Games completed: %d\n", server->completed);
  fprintf(stderr, "Games terminated: %d\n", server->terminated);
  fprintf(stderr, "Total tricks: %d\n", server->tricks);
  fflush(stderr);
  pthread_mutex_unlock(&server->contextLock);
}

/* signal_thread()
 * ---------------
 * Handles signals (e.g., SIGHUP) and prints server statistics.
 *
 * arg: pointer to the ServerContext struct containing server state
 *
 * Returns: NULL
 * Global variables modified: none
 * Errors: none
 */
void *signal_thread(void *arg) {
  ServerContext *server = (ServerContext *)arg;
  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGHUP);

  while (1) {
    int sig;
    sigwait(&set, &sig);
    if (sig == SIGHUP) {
      print_statistics(server);
    }
  }
  return NULL;
}

/* create_client_info()
 * --------------------
 * Creates and initializes a new ClientInfo struct for a client.
 *
 * ctx: pointer to the ServerContext struct containing server state
 * clientFd: the file descriptor of the client socket
 * threadArgs: pointer to the ThreadArgs struct to populate
 * args: pointer to the Arguments struct containing server arguments
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void create_client_info(ServerContext *ctx, int clientFd,
                        ThreadArgs *threadArgs, Arguments *args) {
  ClientInfo *clientInfo = (ClientInfo *)malloc(sizeof(ClientInfo));
  clientInfo->fd = clientFd;
  clientInfo->next = NULL;

  pthread_mutex_lock(&ctx->clientLock);
  clientInfo->next = (struct ClientInfo*)ctx->clientList;
  ctx->clientList = clientInfo;
  pthread_mutex_unlock(&ctx->clientLock);

  threadArgs->fd = clientFd;
  threadArgs->clientInfo = clientInfo;
  threadArgs->ctx = ctx;
  threadArgs->message = args->message;
}

/* accept_new_client()
 * --------------------
 * Accepts a new client connection and starts a thread to handle it.
 *
 * fd: the file descriptor of the server socket
 * ctx: pointer to the ServerContext struct containing server state
 * args: pointer to the Arguments struct containing server arguments
 *
 * Returns: none
 * Global variables modified: none
 * Errors: none
 */
void accept_new_client(int fd, ServerContext *ctx, Arguments *args) {
  sem_wait(&ctx->conn[0]);
  struct sockaddr_in address;
  socklen_t addrlen = sizeof(address);

  int clientFd = accept(fd, (struct sockaddr *)&address, &addrlen);
  if (clientFd < 0) {
    sem_post(&ctx->conn[0]);
    return;
  }

  pthread_mutex_lock(&ctx->contextLock);
  ctx->clientCount++;
  ctx->connected++;
  pthread_mutex_unlock(&ctx->contextLock);

  ThreadArgs *threadArgs = (ThreadArgs *)malloc(sizeof(ThreadArgs));
  create_client_info(ctx, clientFd, threadArgs, args);

  pthread_t id;
  pthread_create(&id, NULL, handle_new_connection, threadArgs);
  pthread_detach(id);
}

/* main()
 * ------
 * The main function for the server, initializing and running the game server.
 *
 * argc: number of command line arguments
 * argv: array of command line argument strings
 *
 * Returns: 0 on successful execution
 * Global variables modified: none
 * Errors: prints usage and exits if arguments are invalid
 */
int main(int argc, char *argv[]) {
  struct sigaction act;
  act.sa_flags = 0;
  act.sa_handler = SIG_IGN;
  sigemptyset(&act.sa_mask);
  sigaction(SIGPIPE, &act, 0);

  Arguments *args = (Arguments *)malloc(sizeof(Arguments));
  parse_command_line_arguments(argc, argv, args);
  const char *port = argc == maxArgsSize ? argv[minArgsSize] : "0";
  int fd = check_port(port);

  ServerContext *ctx = init_server();

  sigset_t set;
  sigemptyset(&set);
  sigaddset(&set, SIGHUP);
  pthread_sigmask(SIG_BLOCK, &set, NULL);

  pthread_t sigThread;
  pthread_create(&sigThread, NULL, signal_thread, ctx);
  pthread_detach(sigThread);

  sem_t connected;
  sem_init(&connected, 0, args->maxconns);
  ctx->conn = &connected;

  while (1) {
    accept_new_client(fd, ctx, args);
  }

  free(args);
  return 0;
}