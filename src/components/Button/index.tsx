import type { ButtonHTMLAttributes } from "react";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
    scheme: "primary" | "secondary";
}

const Button = ({
    children,
    type = "button",
    scheme,
    ...props
}: ButtonProps) => {
    return (
        <button type={type} {...props}>
            {children}
        </button>
    );
};

export default Button;
