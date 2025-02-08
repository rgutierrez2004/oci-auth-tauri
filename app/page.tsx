'use client'

import { Button, Card, CardBody, Chip, Link, Image, Divider, Modal, ModalContent, ModalHeader, ModalBody, ModalFooter, Input } from "@nextui-org/react";
import { title, subtitle } from "@/components/primitives";
import { useTheme } from "next-themes";
import { useState, useEffect, useRef } from "react";
import { attachConsole, info, error, debug, trace, warn } from '@tauri-apps/plugin-log';
import { Store } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';
import { Highlight, themes } from "prism-react-renderer";
import clsx from 'clsx';
import { EyeFilledIcon, EyeSlashFilledIcon } from './icons';

// Log levels matching Rust's log levels
enum LogLevel {
  Trace = 'trace',
  Debug = 'debug',
  Info = 'info',
  Warn = 'warn',
  Error = 'error',
  Off = 'off'
}

interface LoggingConfig {
  level: LogLevel;
}

interface AppConfig {
  logging?: LoggingConfig;
}

interface CauseMessage {
  code: string;
  message: string;
}

interface AuthResponse {
  status: string;
  ecId: string;
  displayName: string;
  nextAuthFactors: string[];
  cause: CauseMessage[];
  nextOp: string[];
  scenario: string;
  requestState: string;
}

const useLogLevel = async (): Promise<LogLevel> => {
  try {
    // Get the full path to config.json from the app config dir
    const store = await Store.load('.config.json');
    const config = await store.get<AppConfig>('config');
    return config?.logging?.level || LogLevel.Info;
  } catch (e: unknown) {
    console.error('Error loading config');
    return LogLevel.Info; // Default to Info if there's an error
  }
};

// Debounce helper
const debounce = <T extends (...args: any[]) => any>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: NodeJS.Timeout;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
};

export default function Home() {
  const { theme, setTheme } = useTheme();
  const [activeSection, setActiveSection] = useState("home");
  const [isLoginOpen, setIsLoginOpen] = useState(false);
  const [isPasswordVisible, setIsPasswordVisible] = useState(false);
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [requestState, setRequestState] = useState<string>("");
  const [authMessage, setAuthMessage] = useState<string>("");
  const [showAuthMessage, setShowAuthMessage] = useState(false);
  const [mockUserData, setMockUserData] = useState<any>({});
  const isManualScrollRef = useRef(false);

  // Initialize logging once when component mounts
  useEffect(() => {
    const initLogging = async () => {
      const logLevel = await useLogLevel();

      try {
        // Attach console to send logs to both stdout and file
        await attachConsole();
        if (logLevel === LogLevel.Info) {
          info('Application UI ready');
        }
        if (logLevel === LogLevel.Debug) {
          debug('Console attached, logging to stdout and file');
        }
      } catch (e) {
        console.error('Failed to initialize logging:', e);
        if (logLevel === LogLevel.Error) {
          error('Failed to initialize logging');
        }
      }
    };

    initLogging();
  }, []);

  useEffect(() => {
    const handleScroll = async () => {
      const logLevel = await useLogLevel();

      if (isManualScrollRef.current) {
        if (logLevel === LogLevel.Trace) {
          trace('Ignoring scroll event due to manual scroll');
        }        
        isManualScrollRef.current = false;
        return;
      }

      const sections = document.querySelectorAll('section');
      let currentSection = '';

      sections.forEach((section) => {
        const rect = section.getBoundingClientRect();
        if (rect.top <= 100 && rect.bottom >= 100) {
          currentSection = section.id;
        }
      });

      if (currentSection && currentSection !== activeSection) {
        if (logLevel === LogLevel.Info) {
          info(`Navigated to section: ${currentSection}`);
        }
        if (logLevel === LogLevel.Debug) {
          debug(`Navigation details - href: #${currentSection}`);
        }
        setActiveSection(currentSection);
      } else if (!currentSection && activeSection) {
        if (logLevel === LogLevel.Info) {
          info('Returned to top of page');
        }        
        setActiveSection('');
      }
    };

    const safeHandleScroll = () => {
      handleScroll().catch(error);
    };

    const debouncedHandleScroll = debounce(safeHandleScroll, 100);

    window.addEventListener('scroll', debouncedHandleScroll);

    return () => {
      window.removeEventListener('scroll', debouncedHandleScroll);
    };
  }, [activeSection]);

  const handleSignIn = async (onClose: () => void) => {
    try {
      const response = await invoke<AuthResponse>('initiate_auth', { username, password });
      const message = response.cause?.[0]?.message || 'Authentication in progress...';
      setAuthMessage(message);
      setRequestState(response.requestState);
      onClose(); // Close login modal before showing auth message
      setShowAuthMessage(true);
    } catch (error) {
      console.error('Authentication error:', error);
      setAuthMessage('Authentication failed. Please try again.');
      setShowAuthMessage(true);
      // Reset login fields on error
      setUsername("");
      setPassword("");
    }
  };

  const handleAuthMessageConfirm = async () => {
    // If requestState is empty, just close the modal
    if (!requestState) {
      setShowAuthMessage(false);
      // Reset login fields if there was an error
      setUsername("");
      setPassword("");
      return;
    }

    try {
      // Complete the authentication
      const userProfile = await invoke<any>("complete_auth", {
        requestState
      });
      
      // Update authentication state
      setIsAuthenticated(true);
      setShowAuthMessage(false);
      
      // Update user data
      setMockUserData(userProfile);
      
      // Navigate to user section after login
      handleNavigation("user");
      
    } catch (error) {
      setAuthMessage(typeof error === 'string' ? error : 'Authentication error.');
      // Clear request state so next OK click just closes the modal
      setRequestState("");
      // Reset login fields on error
      setUsername("");
      setPassword("");
    }
  };

  const handleSignOff = () => {
    setIsAuthenticated(false);
    setUsername("");
    setPassword("");
    setMockUserData({});
    handleNavigation("home");
    info("User signed off");
  };

  const handleNavigation = (section: string) => {
    const logLevel = LogLevel.Info; // Default to Info if not loaded yet
    
    if (section === 'user' && !isAuthenticated) {
      if (logLevel === LogLevel.Info) {
        info('Attempted to access user section without authentication');
      }
      return;
    }

    const element = document.getElementById(section);
    if (element) {
      isManualScrollRef.current = true;
      element.scrollIntoView({ behavior: 'smooth' });
      if (logLevel === LogLevel.Info) {
        info(`Navigated to section: ${section}`);
      }
      setActiveSection(section);
    }
  };

  const handleClick = async (e: React.MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault();

    const href = e.currentTarget.getAttribute('href');
    if (!href) return;

    const logLevel = await useLogLevel();

    isManualScrollRef.current = true;
    if (logLevel === LogLevel.Info) {
      info(`Navigated to section: ${href.substring(1)}`);
    }
    if (logLevel === LogLevel.Debug) {
      debug(`Navigation details - href: ${href}`);
    }

    const targetId = href.replace(/.*#/, '');
    const elem = document.getElementById(targetId);
    elem?.scrollIntoView({
      behavior: 'smooth'
    });
  };

  const scrollToTop = async () => {
    if (typeof window === 'undefined') {
      warn('Window object not available');
      return;
    }

    const logLevel = await useLogLevel();

    if (logLevel === LogLevel.Debug) {
      debug('Initiating scroll to top');
    }

    isManualScrollRef.current = true;
    window.scrollTo({ top: 0, behavior: 'smooth' });
    setActiveSection("");

    if (logLevel === LogLevel.Info) {
      info('Returned to top of page');
    }
    
    setTimeout(() => {
      isManualScrollRef.current = false;
      if (logLevel === LogLevel.Trace) {
        trace('Manual scroll flag reset');
      }
    }, 1000);
  };

  const NavLink = ({ href, children }: { href: string, children: React.ReactNode }) => {
    const section = href.slice(1);
    const isActive = section === activeSection;
    return (
      <Link
        className={`text-sm font-medium relative group transition-colors duration-300 px-1 py-0.5 ${
          isActive 
            ? 'text-primary font-semibold' 
            : 'text-muted hover:text-foreground'
        }`}
        href={href}
        onClick={(e) => {
          e.preventDefault();
          handleNavigation(section);
        }}
      >
        {children}
        <span
          className={`absolute left-0 right-0 bottom-0 h-px bg-primary transform origin-left transition-transform duration-300 ${
            isActive
              ? 'scale-x-100'
              : 'scale-x-0 opacity-0 group-hover:scale-x-100 group-hover:opacity-100'
          }`} 
        />
      </Link>
    );
  };

  return (
    <div className="relative flex flex-col h-screen">
      {/* Header */}
      <header className="fixed top-0 left-0 right-0 z-50 flex justify-between items-center py-4 px-6 bg-background border-b border-zinc-200 dark:border-zinc-800">
        <div className="flex items-center gap-2">
          <Link
            href="#"
            onClick={() => handleNavigation("home")}
            className="font-bold text-inherit"
          >
            OCI Auth Tauri
          </Link>
        </div>
        <div className="flex-1 flex justify-center">
          <nav className="hidden sm:flex gap-6 items-center border rounded-full px-8 py-2.5 border-zinc-200 dark:border-zinc-800 bg-background/50 backdrop-blur-sm shadow-sm">
            <NavLink href="#features">Features</NavLink>
            <NavLink href="#pricing">Pricing</NavLink>
            <NavLink href="#about">About</NavLink>
            {isAuthenticated && <NavLink href="#user">User</NavLink>}
          </nav>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="light"
            className="text-sm font-normal text-muted"
            onPress={() => {
              if (isAuthenticated) {
                handleSignOff();
              } else {
                setUsername("");
                setPassword("");
                setIsLoginOpen(true);
              }
            }}
          >
            {isAuthenticated ? "Sign out" : "Sign in"}
          </Button>
          <Button
            isIconOnly
            variant="light"
            onClick={async () => {
              debug('Theme toggle clicked');
              const newTheme = theme === 'dark' ? 'light' : 'dark';
              debug(`Switching theme to: ${newTheme}`);
              setTheme(newTheme);
              info(`Theme changed to ${newTheme}`);
            }}
            className="text-muted"
          >
            {theme === 'dark' ? (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth={1.5}
                stroke="currentColor"
                className="w-5 h-5"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M12 3v2.25m6.364.386l-1.591 1.591M21 12h-2.25m-.386 6.364l-1.591-1.591M12 18.75V21m-4.773-4.227l-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 11-7.5 0 3.75 3.75 0 017.5 0z"
                />
              </svg>
            ) : (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth={1.5}
                stroke="currentColor"
                className="w-5 h-5"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M21.752 15.002A9.718 9.718 0 0118 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 003 11.25C3 16.635 7.365 21 12 21a9.753 9.753 0 009.002-5.998z"
                />
              </svg>
            )}
          </Button>
        </div>
      </header>

      <main className="flex-grow pt-[72px]"> 
        {/* User Section */}
        <section 
          id="user" 
          className={clsx(
            "w-full py-12 px-6",
            { "hidden": activeSection !== "user" }
          )}
        >
          <div className="container mx-auto max-w-[1280px]">
            <Card className="py-4 w-full">
              <CardBody>
                <div className="flex flex-col gap-4">
                  <h2 className={title({ size: "sm" })}>User Profile</h2>
                  <Divider/>
                  <div>
                    <Highlight
                      theme={theme === 'dark' ? themes.nightOwl : themes.github}
                      code={JSON.stringify(mockUserData, null, 2)}
                      language="json"
                    >
                      {({ className, style, tokens, getLineProps, getTokenProps }) => (
                        <pre className={className + " p-4 text-sm whitespace-pre-wrap break-all"} style={style}>
                          {tokens.map((line, i) => (
                            <div key={i} {...getLineProps({ line })}>
                              <span className="select-none opacity-50 mr-4">{i + 1}</span>
                              {line.map((token, key) => (
                                <span key={key} {...getTokenProps({ token })} />
                              ))}
                            </div>
                          ))}
                        </pre>
                      )}
                    </Highlight>
                  </div>
                </div>
              </CardBody>
            </Card>
          </div>
        </section>

        {/* Hero Section */}
        <section
          id="home"
          className={clsx(
            "w-full py-12 md:py-24 lg:py-32 xl:py-48",
            { "hidden": activeSection !== "home" }
          )}
        >
          <div className="container mx-auto px-6 max-w-[1024px]">
            <div className="flex flex-col items-center text-center gap-4">
              <Chip
                variant="flat"
                className="bg-zinc-100 dark:bg-zinc-800 text-muted"
              >
                ✨ Your OCI Authentication Companion
              </Chip>
              <h1 className={title({ class: "max-w-[800px]" })}>
                Secure OCI Authentication
                <br />
                <span className="text-gradient">Made Simple</span>
              </h1>
              <p className={subtitle({ class: "max-w-[600px] text-muted" })}>
                Streamline your Oracle Cloud Infrastructure authentication process with our powerful desktop application.
              </p>
              <div className="flex gap-3 mt-8">
                <Button
                  size="lg"
                  radius="full"
                  variant="bordered"
                  className="border-zinc-200 dark:border-zinc-800 text-muted"
                >
                  Live Demo
                </Button>
              </div>
              <div className="mt-8 p-8 bg-zinc-100/50 dark:bg-zinc-800/50 rounded-xl backdrop-blur-sm">
                <Image
                  src="/app-preview.png"
                  alt="App preview"
                  width={800}
                  height={400}
                  className="rounded-lg shadow-xl"
                />
              </div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section
          id="features"
          className={clsx(
            "flex flex-col items-center justify-center gap-4 py-8 md:py-10",
            { "hidden": activeSection !== "features" }
          )}
        >
          <div className="container mx-auto px-6">
            <div className="text-center mb-16">
              <h2 className={title({ size: "sm" })}>Features</h2>
              <p className={subtitle({ class: "mx-auto text-muted" })}>
                Everything you need for secure OCI authentication
              </p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
              {[
                {
                  icon: "🔐",
                  title: "Secure by Default",
                  description: "Enterprise-grade security for your credentials"
                },
                {
                  icon: "⚡",
                  title: "Lightning Fast",
                  description: "Instant authentication with minimal overhead"
                },
                {
                  icon: "🎯",
                  title: "User Friendly",
                  description: "Intuitive interface for seamless experience"
                },
                {
                  icon: "🔄",
                  title: "Auto Sync",
                  description: "Automatic profile synchronization"
                },
                {
                  icon: "📱",
                  title: "Cross Platform",
                  description: "Works on Windows, macOS, and Linux"
                },
                {
                  icon: "🛡️",
                  title: "Token Management",
                  description: "Advanced token lifecycle management"
                }
              ].map((feature, index) => (
                <Card key={index} className="bg-zinc-100/50 dark:bg-zinc-800/50 backdrop-blur-sm">
                  <CardBody className="text-center p-6">
                    <div className="mb-4 text-4xl">{feature.icon}</div>
                    <h3 className="text-large font-bold mb-2">{feature.title}</h3>
                    <p className="text-small text-muted">{feature.description}</p>
                  </CardBody>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {/* Pricing Section */}
        <section
          id="pricing"
          className={clsx(
            "flex flex-col items-center justify-center gap-4 py-8 md:py-10",
            { "hidden": activeSection !== "pricing" }
          )}
        >
          <div className="container mx-auto px-6">
            <div className="text-center mb-16">
              <h2 className={title({ size: "sm" })}>Simple Pricing</h2>
              <p className={subtitle({ class: "mx-auto text-muted" })}>
                Choose the plan that works best for you
              </p>
            </div>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-5xl mx-auto">
              {[
                {
                  name: "Free",
                  price: "$0",
                  description: "Perfect for individual developers",
                  features: [
                    "Basic authentication",
                    "Single profile",
                    "Community support"
                  ]
                },
                {
                  name: "Pro",
                  price: "$9",
                  description: "Great for professional developers",
                  features: [
                    "Advanced authentication",
                    "Multiple profiles",
                    "Priority support",
                    "Token management"
                  ]
                },
                {
                  name: "Enterprise",
                  price: "Custom",
                  description: "For large organizations",
                  features: [
                    "Custom authentication flows",
                    "Unlimited profiles",
                    "24/7 support",
                    "Advanced security features",
                    "Custom integration"
                  ]
                }
              ].map((plan, index) => (
                <Card key={index} className="bg-background">
                  <CardBody className="p-6">
                    <div className="text-center mb-4">
                      <h3 className="text-large font-bold">{plan.name}</h3>
                      <div className="text-2xl font-bold mt-2">{plan.price}</div>
                      <p className="text-small text-muted mt-2">{plan.description}</p>
                    </div>
                    <Divider className="my-4" />
                    <ul className="space-y-3">
                      {plan.features.map((feature, i) => (
                        <li key={i} className="flex items-center text-small">
                          <svg className="w-4 h-4 mr-2 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                          </svg>
                          {feature}
                        </li>
                      ))}
                    </ul>
                    <Button
                      className="w-full mt-6"
                      color="primary"
                      variant={index === 1 ? "solid" : "bordered"}
                      size="lg"
                    >
                      Get Started
                    </Button>
                  </CardBody>
                </Card>
              ))}
            </div>
          </div>
        </section>

        {/* About Section */}
        <section
          id="about"
          className={clsx(
            "flex flex-col items-center justify-center gap-4 py-8 md:py-10",
            { "hidden": activeSection !== "about" }
          )}
        >
          <div className="container mx-auto px-6">
            <div className="max-w-3xl mx-auto text-center">
              <h2 className={title({ size: "sm" })}>About OCI Auth Tauri</h2>
              <p className={subtitle({ class: "mx-auto mt-4 text-muted" })}>
                We're on a mission to simplify OCI authentication for developers worldwide. Our team of security experts and developers has created a robust solution that makes authentication both secure and effortless.
              </p>
              <div className="mt-8 grid grid-cols-1 md:grid-cols-3 gap-8">
                <div>
                  <div className="text-4xl font-bold text-primary">10k+</div>
                  <div className="text-sm text-muted mt-2">Active Users</div>
                </div>
                <div>
                  <div className="text-4xl font-bold text-primary">99.9%</div>
                  <div className="text-sm text-muted mt-2">Uptime</div>
                </div>
                <div>
                  <div className="text-4xl font-bold text-primary">24/7</div>
                  <div className="text-sm text-muted mt-2">Support</div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* CTA Section */}
        <section
          id="cta"
          className={clsx(
            "flex flex-col items-center justify-center gap-4 py-8 md:py-10",
            { "hidden": activeSection !== "cta" }
          )}
        >
          <div className="container mx-auto px-6">
            <div className="max-w-3xl mx-auto text-center">
              <h2 className={title({ size: "sm" })}>Ready to get started?</h2>
              <p className={subtitle({ class: "mx-auto mt-4 text-muted" })}>
                Join thousands of developers who trust our solution for OCI authentication
              </p>
              <div className="flex justify-center gap-3 mt-8">
                <Button
                  size="lg"
                  radius="full"
                  variant="bordered"
                  className="border-zinc-200 dark:border-zinc-800 text-muted"
                >
                  View Documentation
                </Button>
              </div>
            </div>
          </div>
        </section>
      </main>

      {/* Authentication Message Modal */}
      {showAuthMessage && (
        <Modal 
          isOpen={showAuthMessage} 
          onClose={() => setShowAuthMessage(false)}
          placement="center"
        >
          <ModalContent>
            <ModalHeader className="flex flex-col gap-1">Authentication Message</ModalHeader>
            <ModalBody>
              <p>{authMessage}</p>
            </ModalBody>
            <ModalFooter>
              <Button color="primary" onPress={handleAuthMessageConfirm}>
                OK
              </Button>
            </ModalFooter>
          </ModalContent>
        </Modal>
      )}

      {/* Login Modal */}
      <Modal 
        isOpen={isLoginOpen} 
        onOpenChange={(isOpen) => {
          if (isOpen) {
            setUsername("");
            setPassword("");
          }
          setIsLoginOpen(isOpen);
        }}
        placement="center"
      >
        <ModalContent>
          <ModalHeader className="flex flex-col gap-1">Sign In</ModalHeader>
          <ModalBody>
            <form className="flex flex-col gap-4">
              <Input
                label="Username"
                placeholder="Enter your username"
                value={username}
                onValueChange={setUsername}
              />
              <Input
                label="Password"
                type={isPasswordVisible ? "text" : "password"}
                placeholder="Enter your password"
                value={password}
                onValueChange={setPassword}
                endContent={
                  <button type="button" onClick={() => setIsPasswordVisible(!isPasswordVisible)}>
                    {isPasswordVisible ? (
                      <EyeSlashFilledIcon className="text-2xl text-default-400 pointer-events-none" />
                    ) : (
                      <EyeFilledIcon className="text-2xl text-default-400 pointer-events-none" />
                    )}
                  </button>
                }
              />
            </form>
          </ModalBody>
          <ModalFooter>
            <Button color="danger" variant="light" onPress={() => setIsLoginOpen(false)}>
              Close
            </Button>
            <Button color="primary" onPress={() => handleSignIn(() => setIsLoginOpen(false))}>
              Sign in
            </Button>
          </ModalFooter>
        </ModalContent>
      </Modal>

      {/* Footer */}
      <footer className="w-full py-6 px-6 border-t border-zinc-200 dark:border-zinc-800">
        <div className="container mx-auto flex flex-col md:flex-row justify-between items-center gap-4">
          <div className="text-small text-muted">
            2025 OCI Auth Tauri. All rights reserved.
          </div>
          <div className="flex gap-4">
            <Link href="#" size="sm" className="text-muted">Privacy Policy</Link>
            <Link href="#" size="sm" className="text-muted">Terms of Service</Link>
            <Link href="#" size="sm" className="text-muted">Contact</Link>
          </div>
        </div>
      </footer>
    </div>
  );
}
