import { AnimatePresence, motion, stagger } from "motion/react";
import logo from "../../assets/chess-logo.svg";
import Button from "../Button";
import { useNavigate } from "react-router";

const Menu = ({ className = "" }) => {
    let navigate = useNavigate();

    return (
        <div
            className={`h-full w-full flex justify-center items-center flex-col gap-10 ${className}`}
        >
            <motion.div
                className="logo"
                initial={{ opacity: 0, y: "-50%" }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.5, type: "spring" }}
                exit={{ opacity: 0, y: "100%" }}
            >
                <img src={logo} alt="logo" className="" />
            </motion.div>

            <div className="buttons flex-col flex gap-5">
                <motion.div
                    initial={{ opacity: 0, x: "50%" }}
                    animate={{
                        opacity: 1,
                        x: 0,
                    }}
                    exit={{ opacity: 0, y: "100%" }}
                >
                    <Button
                        scheme="primary"
                        onClick={() => navigate("/game")}
                        className=""
                    >
                        Local PVP
                    </Button>
                </motion.div>
                <motion.div
                    initial={{ opacity: 0, x: "-50%" }}
                    animate={{
                        opacity: 1,
                        x: 0,
                    }}
                    exit={{ opacity: 0, y: "100%" }}
                >
                    <Button
                        scheme="secondary"
                        onClick={() => navigate("/game")}
                        className=""
                    >
                        Against Ai
                    </Button>
                </motion.div>
            </div>
        </div>
    );
};

export default Menu;
