/** Shared, so it's only loaded once; created lazily, as there is no `Audio` outside the browser. */
let todoCompletedAudio: HTMLAudioElement | null = null;

/** Plays a short bell, rewarding the completion of a todo. */
export const playTodoCompletedSound = () => {
  if (!todoCompletedAudio) {
    todoCompletedAudio = new Audio("/sounds/todo-completed.wav");
    todoCompletedAudio.volume = 0.6;
  }
  // Restarts it when completing several todos in a row
  todoCompletedAudio.currentTime = 0;
  // A sound failing to play (e.g. no output device) must not break the completion
  todoCompletedAudio.play().catch(() => {});
};
