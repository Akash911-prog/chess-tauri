import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";

type Props = {
    isOpen: boolean;
    title: string;
    message: string;
    onRestart: () => void;
    onMainMenu: () => void;
};

/**
 * GameEndModal
 *
 * Controlled modal — visibility lives in the parent's state.
 * Drop in `isOpen`, wire up the two callbacks, swap the placeholder text.
 *
 * <GameEndModal
 *   isOpen={showModal}
 *   title="CHECKMATE"
 *   message="White wins."
 *   onRestart={() => {...}}
 *   onMainMenu={() => {...}}
 * />
 */
function GameOverModal({
    isOpen,
    title = "TITLE_PLACEHOLDER",
    message = "message_placeholder_goes_here",
    onRestart,
    onMainMenu,
}: Props) {
    return (
        <AnimatePresence>
            {isOpen && (
                <motion.div
                    className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4"
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    transition={{ duration: 0.15 }}
                >
                    <motion.div
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="game-end-title"
                        className="w-full max-w-sm border-2 border-amber-500 bg-neutral-950 font-mono text-amber-100 shadow-[6px_6px_0_0_rgba(245,158,11,0.4)] h-[200px] flex flex-col justify-between"
                        initial={{ opacity: 0, scale: 0.85, y: 12 }}
                        animate={{ opacity: 1, scale: 1, y: 0 }}
                        exit={{ opacity: 0, scale: 0.9, y: 8 }}
                        transition={{
                            type: "spring",
                            stiffness: 420,
                            damping: 26,
                        }}
                    >
                        {/* header */}
                        <div className="border-b-2 border-amber-500/40 px-5 py-3 h-[30%] w-full">
                            <p className="text-[10px] tracking-[0.3em] text-amber-500/70 relative left-5 top-2">
                                GAME_OVER
                            </p>
                            <h2
                                id="game-end-title"
                                className="mt-1 text-xl font-bold tracking-wide text-amber-400 relative left-5 top-2"
                            >
                                {title}
                            </h2>
                        </div>

                        {/* body */}
                        <div className="px-5 py-4">
                            <p className="text-sm leading-relaxed text-amber-100/80 mx-auto! w-fit">
                                {message}
                            </p>
                        </div>

                        {/* actions */}
                        <div className="grid grid-cols-2 border-t-2 border-amber-500/40 h-10">
                            <button
                                type="button"
                                onClick={onMainMenu}
                                className="border-r-2 border-amber-500/40 px-4 py-3 text-xs font-bold tracking-[0.2em] text-amber-100/70 transition-colors hover:bg-amber-500/10 hover:text-amber-100 focus:outline-none focus-visible:bg-amber-500/10"
                            >
                                MAIN MENU
                            </button>
                            <button
                                type="button"
                                onClick={onRestart}
                                className="bg-amber-500 px-4 py-3 text-xs font-bold tracking-[0.2em] text-neutral-950 transition-colors hover:bg-amber-400 focus:outline-none focus-visible:bg-amber-400"
                            >
                                RESTART
                            </button>
                        </div>
                    </motion.div>
                </motion.div>
            )}
        </AnimatePresence>
    );
}

export default GameOverModal;
