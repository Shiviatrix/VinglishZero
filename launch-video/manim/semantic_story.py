"""Minimal semantic-proof sequences for the Vinglish Zero launch film."""

from pathlib import Path

from manimpango import register_font

from manim import (
    DOWN,
    LEFT,
    ORIGIN,
    RIGHT,
    UP,
    AnimationGroup,
    Circle,
    Create,
    FadeIn,
    FadeOut,
    LaggedStart,
    Rectangle,
    Scene,
    Text,
    Transform,
    VGroup,
    Write,
    config,
)

config.pixel_width = 3840
config.pixel_height = 2160
config.frame_rate = 30

FONT_DIRECTORY = Path(__file__).parent / "fonts"
for font_file in ("Baloo2.ttf", "SpaceMono-Regular.ttf", "SpaceMono-Bold.ttf"):
    path = FONT_DIRECTORY / font_file
    if path.exists():
        register_font(str(path))

# Match the Remotion matte-black and pastel palette.
CREAM = "#0E0D0C"
PAPER = "#E8E2D9"
RED = "#E97866"
BLUE = "#8AB6C9"
GREEN = "#9BBF9A"
ORANGE = "#E89143"
PINK = "#D7A0A8"
INK = "#171513"
TEXT = "#F4F0E9"
MUTED = "#B9B1A7"
YELLOW = "#E4CC76"

TEAL = BLUE
VIOLET = PINK
AMBER = ORANGE
def word(text, size=34, color=TEXT, weight="NORMAL"):
    return Text(text, font="Baloo 2", font_size=size, color=color, weight=weight)


def mono(text, size=24, color=TEXT, weight="NORMAL"):
    return Text(text, font="Space Mono", font_size=size, color=color, weight=weight)


def accent_text_color(color):
    # Every accent is intentionally pastel, so dark text has the best contrast.
    return INK


def stamped_panel(width, height, fill=PAPER):
    # Keep a transparent placeholder at index 0 so existing panel content can
    # retain its face-at-index-1 layout without any visible drop shadow.
    shadow = Rectangle(width=width, height=height, stroke_width=0, fill_opacity=0)
    face = Rectangle(
        width=width,
        height=height,
        stroke_color=MUTED,
        stroke_width=3,
        fill_color=fill,
        fill_opacity=1,
    )
    return VGroup(shadow, face)


def chip(label, color, width=None, font_size=27):
    text = mono(label, font_size, accent_text_color(color), "BOLD")
    panel_width = width or max(2.2, text.width + 0.7)
    panel = stamped_panel(panel_width, max(0.8, text.height + 0.44), color)
    text.move_to(panel[1].get_center())
    return VGroup(panel, text)


def paper_chip(label, width=None, font_size=27):
    return chip(label, PAPER, width, font_size)


class FilmScene(Scene):
    def setup(self):
        self.camera.background_color = CREAM

    def heading(self, top, bottom=None):
        main = word(top, 56, TEXT, "BOLD").to_edge(UP, buff=0.58)
        if bottom is None:
            return main
        sub = mono(bottom, 22, MUTED, "BOLD").next_to(main, DOWN, buff=0.18)
        return VGroup(main, sub)


class SemanticLift(FilmScene):
    """Show four surface forms converging into one shared semantic pipeline."""

    def construct(self):
        heading = self.heading("Syntax falls away.", "The semantic role remains.")
        self.play(FadeIn(heading, shift=UP * 0.14), run_time=0.7)

        sources = VGroup(
            chip("PYTHON", TEAL, 2.55),
            chip("JAVA", AMBER, 2.25),
            chip("C", VIOLET, 1.55),
            chip("VINGLISH", GREEN, 2.7),
        ).arrange(RIGHT, buff=0.34).move_to(UP * 1.12)
        source_detail = mono("Four surface forms. One behavior.", 21, MUTED, "BOLD").next_to(sources, DOWN, buff=0.34)
        self.play(LaggedStart(*[FadeIn(item, shift=UP * 0.22) for item in sources], lag_ratio=0.12), run_time=1.35)
        self.play(FadeIn(source_detail), run_time=0.38)
        self.wait(0.75)

        graph_panel = stamped_panel(4.65, 2.72, PAPER)
        graph = VGroup(
            graph_panel,
            Circle(radius=0.82, color=TEAL, stroke_width=8),
            Circle(radius=0.56, color=VIOLET, stroke_width=6),
            word("Universal", 29, INK, "BOLD"),
            word("Semantic Graph", 25, INK, "BOLD"),
        )
        graph[1].move_to(graph_panel[1].get_center() + LEFT * 1.15)
        graph[2].move_to(graph[1].get_center())
        graph[3].move_to(graph_panel[1].get_center() + RIGHT * 0.6 + UP * 0.2)
        graph[4].move_to(graph_panel[1].get_center() + RIGHT * 0.6 + DOWN * 0.28)
        graph.move_to(ORIGIN)
        self.play(
            FadeOut(source_detail),
            LaggedStart(*[item.animate.move_to(ORIGIN).scale(0.28).set_opacity(0) for item in sources], lag_ratio=0.08),
            run_time=1.0,
        )
        self.play(Create(graph[0]), Create(graph[1]), Create(graph[2]), Write(graph[3]), Write(graph[4]), run_time=1.15)
        self.wait(0.55)

        stages = VGroup(
            chip("Filter", TEAL, 2.6, 30),
            word("→", 38, INK, "BOLD"),
            chip("Map", VIOLET, 2.25, 30),
            word("→", 38, INK, "BOLD"),
            chip("Reduce", GREEN, 2.75, 30),
        ).arrange(RIGHT, buff=0.32).move_to(DOWN * 2.22)
        caption = mono("Adapter-specific syntax is no longer visible here.", 22, MUTED, "BOLD").next_to(stages, DOWN, buff=0.32)
        self.play(graph.animate.scale(0.68).move_to(UP * 0.3), run_time=0.65)
        self.play(LaggedStart(*[FadeIn(item, shift=DOWN * 0.16) for item in stages], lag_ratio=0.12), run_time=1.25)
        self.play(FadeIn(caption), run_time=0.36)
        self.wait(20)


class ReasoningProof(FilmScene):
    """Visualize facts, evidence, rule evaluation, and a composed report."""

    def construct(self):
        heading = self.heading("Zero runtime guessing.", "Deterministic rules eliminate incompatible hypotheses.")
        self.play(FadeIn(heading, shift=UP * 0.14), run_time=0.7)

        facts = VGroup(
            paper_chip("Loop", 1.95, 22),
            paper_chip("Collection Iteration", 3.7, 22),
            paper_chip("Conditional", 2.4, 22),
            paper_chip("Return", 1.9, 22),
            paper_chip("Call", 1.6, 22),
        ).arrange(RIGHT, buff=0.2).scale(0.8).move_to(UP * 0.72)
        facts_label = mono("Immutable Facts", 22, MUTED, "BOLD").next_to(facts, UP, buff=0.28)
        self.play(FadeIn(facts_label), LaggedStart(*[FadeIn(item, shift=UP * 0.16) for item in facts], lag_ratio=0.1), run_time=1.2)
        self.wait(0.5)

        evidence = VGroup(
            chip("Predicate", TEAL, 2.45, 26),
            chip("Transformation", VIOLET, 3.2, 26),
            chip("Combination", GREEN, 2.8, 26),
        ).arrange(RIGHT, buff=0.42).move_to(DOWN * 0.65)
        evidence_label = mono("Evidence", 22, MUTED, "BOLD").next_to(evidence, UP, buff=0.28)
        self.play(
            FadeOut(facts_label),
            LaggedStart(*[item.animate.scale(0.35).set_opacity(0) for item in facts], lag_ratio=0.07),
            FadeIn(evidence_label),
            LaggedStart(*[FadeIn(item, shift=UP * 0.16) for item in evidence], lag_ratio=0.15),
            run_time=1.2,
        )
        self.wait(0.5)

        active = VGroup(
            chip("Filter · 100", TEAL, 3.3, 23),
            chip("Mapper · 100", VIOLET, 3.55, 23),
            chip("Reducer · 100", GREEN, 3.65, 23),
        ).arrange(RIGHT, buff=0.3).move_to(DOWN * 2.15)
        rejected = chip("Average · Rejected: Division Absent", RED, 5.45, 18).move_to(DOWN * 3.07)
        self.play(
            FadeOut(evidence_label),
            evidence.animate.scale(0.72).move_to(UP * 0.1),
            LaggedStart(*[FadeIn(item, shift=DOWN * 0.16) for item in active], lag_ratio=0.14),
            run_time=1.15,
        )
        self.play(FadeIn(rejected), run_time=0.35)
        self.wait(0.6)

        report_panel = stamped_panel(10.2, 1.65, PAPER)
        report = VGroup(
            report_panel,
            mono("Intent Report", 21, BLUE, "BOLD"),
            word("Filter  →  Mapper  →  Reducer", 28, INK, "BOLD"),
        )
        report[1].move_to(report_panel[1].get_center() + UP * 0.34)
        report[2].move_to(report_panel[1].get_center() + DOWN * 0.26)
        report.move_to(DOWN * 0.86)
        self.play(
            FadeOut(evidence),
            FadeOut(rejected),
            active.animate.scale(0.82).move_to(UP * 0.5),
            FadeIn(report, shift=UP * 0.14),
            run_time=0.75,
        )
        self.wait(22)


class DiagnosticReframe(FilmScene):
    """Use the shipped type-mismatch fixture and its real semantic interpretation."""

    def construct(self):
        heading = self.heading("Understand intent.", "Errors change completely.")
        self.play(FadeIn(heading, shift=UP * 0.14), run_time=0.7)

        compiler_panel = stamped_panel(5.7, 1.65, PAPER).move_to(UP * 0.72)
        compiler = VGroup(
            compiler_panel,
            mono("Type Mismatch", 23, RED, "BOLD"),
            word("Incompatible Values", 37, INK, "BOLD"),
        )
        compiler[1].move_to(compiler_panel[1].get_center() + UP * 0.34)
        compiler[2].move_to(compiler_panel[1].get_center() + DOWN * 0.26)
        self.play(FadeIn(compiler, shift=UP * 0.14), run_time=0.85)
        self.wait(0.65)

        role = chip("Accumulator", YELLOW, 3.65, 29).move_to(DOWN * 0.45)
        evidence = VGroup(
            paper_chip("Return", 1.95, 20),
            paper_chip("Accumulation", 2.8, 20),
            paper_chip("Numeric Return", 3.0, 20),
        ).arrange(RIGHT, buff=0.24).scale(0.83).move_to(DOWN * 1.52)
        self.play(Transform(compiler[2].copy(), role), run_time=0.72)
        self.play(LaggedStart(*[FadeIn(item, shift=DOWN * 0.12) for item in evidence], lag_ratio=0.12), run_time=0.85)
        self.wait(0.45)

        explanation = word("Accumulator Type Conflict", 43, INK, "BOLD").move_to(DOWN * 2.3)
        fixes = VGroup(
            chip("Change Accumulator Type", TEAL, 4.4, 20),
            chip("Convert Incoming Value", VIOLET, 4.2, 20),
        ).arrange(RIGHT, buff=0.25).scale(0.8).move_to(DOWN * 2.98)
        self.play(FadeIn(explanation, shift=DOWN * 0.14), run_time=0.5)
        self.play(LaggedStart(*[FadeIn(item, shift=DOWN * 0.1) for item in fixes], lag_ratio=0.12), run_time=0.7)
        self.wait(20)


class DeterminismProof(FilmScene):
    """Show that repeated analysis of an identical graph emits identical reports."""

    def construct(self):
        heading = self.heading("Byte stable.", "Same graph. Same evidence. Same answer.")
        self.play(FadeIn(heading, shift=UP * 0.14), run_time=0.7)

        def report_card(run):
            panel = stamped_panel(5.3, 3.1, PAPER)
            title = mono(run, 19, BLUE, "BOLD")
            intent = word("Filter → Mapper → Reducer", 25, INK, "BOLD")
            digest = mono("SHA-256 · f81f…2b97", 17, INK, "BOLD")
            content = VGroup(title, intent, digest).arrange(DOWN, aligned_edge=LEFT, buff=0.28)
            content.move_to(panel[1].get_center()).align_to(panel[1], LEFT).shift(RIGHT * 0.38)
            return VGroup(panel, content)

        first = report_card("Run 01").move_to(LEFT * 3.2 + DOWN * 0.65)
        second = report_card("Run 02").move_to(RIGHT * 3.2 + DOWN * 0.65)
        self.play(FadeIn(first, shift=UP * 0.16), run_time=0.7)
        self.play(FadeIn(second, shift=UP * 0.16), run_time=0.7)
        self.wait(0.55)

        result = chip("Byte Stable", GREEN, 4.2, 30).move_to(DOWN * 3.1)
        self.play(FadeIn(result, scale=0.84), run_time=0.5)
        self.wait(20)
