import bgImage from "../assets/potential-main-menu-bg/3.jpg";
import Menu from "../components/Menu";

const MainMenu = () => {
    return (
        <main className="relative min-h-screen w-screen overflow-hidden">
            <div
                className="
                absolute inset-0
                bg-cover
                bg-center
                bg-no-repeat
                left-1/2
                -translate-x-1/2
                -z-10
                min-h-screen
                w-screen
                min-w-75
                max-w-350
                "
                style={{
                    backgroundImage: `url(${bgImage})`,
                }}
            />

            <div
                className="
                relative z-10
                min-w-75
                max-w-350
                w-screen
                left-1/2
                -translate-x-1/2
                min-h-screen
                "
            >
                <Menu className="relative top-1/2 translate-y-1/2" />
            </div>
        </main>
    );
};

export default MainMenu;
