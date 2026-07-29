import { HTMLAttributes } from "react";

interface BackgroundProps extends HTMLAttributes<HTMLDivElement> {}

const Background = ({ children, ...props }: BackgroundProps) => {
    return <div {...props}>{children}</div>;
};

export default Background;
