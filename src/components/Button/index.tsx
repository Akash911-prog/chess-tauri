import type { ButtonHTMLAttributes } from "react";
import wood from "../../assets/wood.jpg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    scheme: "primary" | "secondary";
}

const Button = ({
    children,
    type = "button",
    scheme,
    className = "",
    onClick,
    ...props
}: ButtonProps) => {
    return (
        <div
            className="w-fit h-fit rounded-2xl overflow-hidden hover:translate-y-0.5 transition-all duration-200 btn-shadow active:translate-1.5"
            style={{ backgroundImage: `url(${wood})` }}
        >
            <div className="w-98 h-22 rounded-2xl overflow-hidden border-yellow-600 border-4">
                <button
                    type={type}
                    {...props}
                    onClick={onClick}
                    className={`w-96 h-20 flex justify-center items-center text-2xl rounded-2xl border-black border-4 text-blue-300 ${className}`}
                >
                    {children}
                </button>
            </div>
        </div>
    );
};

export default Button;
